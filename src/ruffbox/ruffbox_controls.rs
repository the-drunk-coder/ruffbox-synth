use std::sync::Arc;

use rubato::{FftFixedIn, Resampler};

// crossbeam for the event queue
use crossbeam::atomic::AtomicCell;
use dashmap::DashMap;

use crate::building_blocks::{
    resolve_parameter_value, SampleBuffer, SynthParameterLabel, SynthParameterValue,
};
use crate::ruffbox::{ControlMessage, ScheduledEvent};
use crate::synths::*;

use crate::ruffbox::ScheduledSource;

/// thin wrapper around a scheduled event instasnce
pub struct PreparedInstance<const BUFSIZE: usize, const NCHAN: usize> {
    ev: ScheduledEvent<BUFSIZE, NCHAN>,
    sr: f32,
}

impl<const BUFSIZE: usize, const NCHAN: usize> PreparedInstance<BUFSIZE, NCHAN> {
    pub fn set_instance_parameter(&mut self, par: SynthParameterLabel, val: &SynthParameterValue) {
        self.ev
            .set_param_or_modulator(par, resolve_parameter_value::<BUFSIZE>(par, val, self.sr));
    }
}

enum BufferType {
    Mono,
    Stereo,
}

/// These are the controls, the part which you use in your control thread
/// to control the Ruffbox, trigger playback, etc ...
pub struct RuffboxControls<const BUFSIZE: usize, const NCHAN: usize> {
    // Buffer lengths and types need to be known when initializing sampler instances,
    // which is why unfortunately we need to mirror them here in the controls.
    // Thanks to the magic of DashMap, we can get around having to
    // use &mut self. Maybe one day I'll find out how to make the controls
    // actually stateless, but until then, the interior mutability pattern
    // comes in handy ...
    buffer_counter: AtomicCell<usize>,
    buffer_lengths: DashMap<usize, usize>,
    buffer_types: DashMap<usize, BufferType>,
    freeze_buffer_offset: usize,
    num_live_buffers: usize,
    num_freeze_buffers: usize,
    max_buffers: usize,
    control_q_send: crossbeam::channel::Sender<ControlMessage<BUFSIZE, NCHAN>>,
    now: Arc<AtomicCell<f64>>, // shared reference to global time counter
    pub samplerate: f32,       // finally after all those years ...
}

impl<const BUFSIZE: usize, const NCHAN: usize> RuffboxControls<BUFSIZE, NCHAN> {
    pub(crate) fn new(
        samplerate: f64,
        live_buffers: usize,
        live_buffer_time: f64,
        max_buffers: usize,
        freeze_buffers: usize,
        now: &Arc<AtomicCell<f64>>,
        tx: crossbeam::channel::Sender<ControlMessage<BUFSIZE, NCHAN>>,
    ) -> RuffboxControls<BUFSIZE, NCHAN> {
        // dash map is strange, mutable without mut ...
        let buffer_lengths = DashMap::new();
        let buffer_types = DashMap::new();
        if live_buffers > 0 {
            // create buffer lenghts for live buffers and freeze buffers
            for b in 0..live_buffers + freeze_buffers {
                buffer_lengths.insert(b, (samplerate * live_buffer_time) as usize);
                buffer_types.insert(b, BufferType::Mono);
            }
        }

        RuffboxControls {
            buffer_counter: AtomicCell::new(if live_buffers > 0 {
                live_buffers + freeze_buffers
            } else {
                0
            }),
            freeze_buffer_offset: live_buffers,
            num_live_buffers: live_buffers,
            num_freeze_buffers: freeze_buffers,
            buffer_lengths,
            buffer_types,
            max_buffers,
            control_q_send: tx,
            samplerate: samplerate as f32,
            now: Arc::clone(now),
        }
    }

    /// prepare a sound source instance, return instance id
    pub fn prepare_instance(
        &self,
        src_type: SynthType,
        timestamp: f64,
        sample_buf: usize,
    ) -> Option<PreparedInstance<BUFSIZE, NCHAN>> {
        Some(PreparedInstance {
            sr: self.samplerate,
            ev: match src_type {
                SynthType::SingleOscillator(osc_type, lp_type, hp_type) => ScheduledEvent::new(
                    timestamp,
                    ScheduledSource::Channel(Box::new(SingleOscillatorSynth::new(
                        osc_type,
                        lp_type,
                        hp_type,
                        self.samplerate,
                    ))),
                ),
                SynthType::RissetBell => ScheduledEvent::new(
                    timestamp,
                    ScheduledSource::Channel(Box::new(RissetBell::new(self.samplerate))),
                ),
                SynthType::Sampler(hpf_type, pf1_type, pf2_type, lpf_type) => ScheduledEvent::new(
                    timestamp,
                    // insert the right sampler type
                    match *self.buffer_types.get(&sample_buf).unwrap() {
                        BufferType::Mono => {
                            ScheduledSource::Channel(Box::new(NChannelSampler::with_bufnum_len(
                                sample_buf,
                                *self.buffer_lengths.get(&sample_buf).unwrap(),
                                hpf_type,
                                pf1_type,
                                pf2_type,
                                lpf_type,
                                self.samplerate,
                            )))
                        }
                        BufferType::Stereo => ScheduledSource::Channel(Box::new(
                            NChannelStereoSampler::with_bufnum_len(
                                sample_buf,
                                *self.buffer_lengths.get(&sample_buf).unwrap(),
                                hpf_type,
                                pf1_type,
                                pf2_type,
                                lpf_type,
                                self.samplerate,
                            ),
                        )),
                    },
                ),
                SynthType::AmbisonicSampler(hpf_type, pf1_type, pf2_type, lpf_type) => {
                    ScheduledEvent::new(
                        timestamp,
                        // insert the right sampler type
                        // only mono sources are spatialized to ambisonic so far ...
                        match *self.buffer_types.get(&sample_buf).unwrap() {
                            BufferType::Mono => ScheduledSource::Ambi(Box::new(
                                AmbisonicSamplerO1::with_bufnum_len(
                                    sample_buf,
                                    *self.buffer_lengths.get(&sample_buf).unwrap(),
                                    hpf_type,
                                    pf1_type,
                                    pf2_type,
                                    lpf_type,
                                    self.samplerate,
                                ),
                            )),
                            // just ignore for now ...
                            BufferType::Stereo => ScheduledSource::Channel(Box::new(
                                NChannelStereoSampler::with_bufnum_len(
                                    sample_buf,
                                    *self.buffer_lengths.get(&sample_buf).unwrap(),
                                    hpf_type,
                                    pf1_type,
                                    pf2_type,
                                    lpf_type,
                                    self.samplerate,
                                ),
                            )),
                        },
                    )
                }
                SynthType::LiveSampler(hpf_type, pf1_type, pf2_type, lpf_type)
                    if self.num_live_buffers > 0 =>
                {
                    let final_bufnum = if sample_buf < self.num_live_buffers {
                        sample_buf
                    } else {
                        0
                    };
                    ScheduledEvent::new(
                        timestamp,
                        ScheduledSource::Channel(Box::new(NChannelSampler::with_bufnum_len(
                            final_bufnum,
                            *self.buffer_lengths.get(&final_bufnum).unwrap(),
                            hpf_type,
                            pf1_type,
                            pf2_type,
                            lpf_type,
                            self.samplerate,
                        ))),
                    )
                }
                SynthType::FrozenSampler(hpf_type, pf1_type, pf2_type, lpf_type)
                    if self.num_freeze_buffers > 0 =>
                {
                    let final_bufnum = if sample_buf < self.num_freeze_buffers {
                        sample_buf + self.freeze_buffer_offset
                    } else {
                        self.freeze_buffer_offset
                    };
                    ScheduledEvent::new(
                        timestamp,
                        ScheduledSource::Channel(Box::new(NChannelSampler::with_bufnum_len(
                            final_bufnum,
                            *self.buffer_lengths.get(&final_bufnum).unwrap(),
                            hpf_type,
                            pf1_type,
                            pf2_type,
                            lpf_type,
                            self.samplerate,
                        ))),
                    )
                }
                _ => {
                    return None;
                } // jump out
            },
        })
    }

    pub fn set_master_parameter(&self, par: SynthParameterLabel, val: SynthParameterValue) {
        self.control_q_send
            .send(ControlMessage::SetGlobalParamOrModulator(
                par,
                resolve_parameter_value(par, &val, self.samplerate),
            ))
            .unwrap();
    }

    /// triggers a synth for buffer reference or a synth
    pub fn trigger(&self, instance: PreparedInstance<BUFSIZE, NCHAN>) {
        self.control_q_send
            .send(ControlMessage::ScheduleEvent(instance.ev))
            .unwrap();
    }

    /// get the current timestamp
    pub fn get_now(&self) -> f64 {
        // this might cause locking on platforms where AtomicCell<float> isn't lockfree
        self.now.load()
    }

    /// transfer contents of live buffer to freeze buffer
    pub fn freeze_buffer(&self, freezbuf: usize, inbuf: usize) {
        // acutal buffer numbers are calculated here ...
        self.control_q_send
            .send(ControlMessage::FreezeBuffer(
                freezbuf + self.freeze_buffer_offset,
                inbuf,
            ))
            .unwrap();
    }

    /// Loads a mono sample and returns the assigned buffer number.
    ///
    /// Resample to current samplerate if necessary and specified.
    /// The sample buffer is passed as mutable because the method adds
    /// interpolation samples without the need of a copy.
    pub fn load_mono_sample(&self, samples: &mut Vec<f32>, resample: bool, sr: f32) -> usize {
        let buffer_id = self.buffer_counter.fetch_add(1);

        if buffer_id > self.max_buffers {
            println!("warning, this buffer won't be loaded, as the maximum allowed number of buffers has been reached!");
            return buffer_id;
        }

        let (buflen, buffer) = if resample && (self.samplerate != sr) {
            // zero-pad for resampling blocks
            if (samples.len() as f32 % 1024.0) > 0.0 {
                let diff = 1024 - (samples.len() % 1024);
                samples.append(&mut vec![0.0; diff]);
            }

            let mut samples_resampled: Vec<f32> = Vec::new();
            let mut resampler =
                FftFixedIn::<f32>::new(sr as usize, self.samplerate as usize, 1024, 1, 1);

            // interpolation samples
            samples_resampled.push(0.0);
            let num_chunks = samples.len() / 1024;
            for chunk in 0..num_chunks {
                let chunk = vec![samples[(1024 * chunk)..(1024 * (chunk + 1))].to_vec()];
                let mut waves_out = resampler.process(&chunk).unwrap();
                samples_resampled.append(&mut waves_out[0]);
            }
            // interpolation samples
            samples_resampled.push(0.0);
            samples_resampled.push(0.0);
            (samples_resampled.len() - 3, samples_resampled)
        } else {
            samples.insert(0, 0.0); // interpolation sample
            samples.push(0.0);
            samples.push(0.0);
            (samples.len() - 3, samples.to_vec())
        };

        self.buffer_lengths.insert(buffer_id, buflen);
        self.buffer_types.insert(buffer_id, BufferType::Mono);
        self.control_q_send
            .send(ControlMessage::LoadSample(
                buffer_id,
                buflen,
                SampleBuffer::Mono(buffer),
            ))
            .unwrap();
        // return bufnum
        buffer_id
    }

    /// Loads a stereo sample and returns the assigned buffer number.
    ///
    /// Resample to current samplerate if necessary and specified.
    /// The sample buffer is passed as mutable because the method adds
    /// interpolation samples without the need of a copy.
    pub fn load_stereo_sample(
        &self,
        samples_left: &mut Vec<f32>,
        samples_right: &mut Vec<f32>,
        resample: bool,
        sr: f32,
    ) -> usize {
        let buffer_id = self.buffer_counter.fetch_add(1);

        if buffer_id > self.max_buffers {
            println!("warning, this buffer won't be loaded, as the maximum allowed number of buffers has been reached!");
            return buffer_id;
        }

        if samples_right.len() < samples_left.len() {
            samples_right.append(&mut vec![0.0; samples_left.len() - samples_right.len()]);
        }

        let (buflen, buffer_left, buffer_right) = if resample && (self.samplerate != sr) {
            // zero-pad for resampling blocks
            if (samples_left.len() as f32 % 1024.0) > 0.0 {
                let diff = 1024 - (samples_left.len() % 1024);
                samples_left.append(&mut vec![0.0; diff]);
            }

            let mut samples_left_resampled: Vec<f32> = Vec::new();
            let mut samples_right_resampled: Vec<f32> = Vec::new();
            let mut resampler_left =
                FftFixedIn::<f32>::new(sr as usize, self.samplerate as usize, 1024, 1, 1);
            let mut resampler_right =
                FftFixedIn::<f32>::new(sr as usize, self.samplerate as usize, 1024, 1, 1);

            // interpolation samples
            samples_left_resampled.push(0.0);
            samples_right_resampled.push(0.0);

            let num_chunks = samples_left.len() / 1024;

            for chunk in 0..num_chunks {
                let chunk_left = vec![samples_left[(1024 * chunk)..(1024 * (chunk + 1))].to_vec()];
                let mut waves_out_left = resampler_left.process(&chunk_left).unwrap();
                samples_left_resampled.append(&mut waves_out_left[0]);
                let chunk_right = vec![samples_left[(1024 * chunk)..(1024 * (chunk + 1))].to_vec()];
                let mut waves_out_right = resampler_right.process(&chunk_right).unwrap();
                samples_right_resampled.append(&mut waves_out_right[0]);
            }
            // interpolation samples
            samples_left_resampled.push(0.0);
            samples_right_resampled.push(0.0);
            (
                samples_left_resampled.len() - 3,
                samples_left_resampled,
                samples_right_resampled,
            )
        } else {
            // add interpolation samples
            samples_left.insert(0, 0.0);
            samples_right.insert(0, 0.0);
            samples_left.push(0.0);
            samples_left.push(0.0);
            samples_right.push(0.0);
            samples_right.push(0.0);
            (
                samples_left.len() - 3,
                samples_left.to_vec(),
                samples_right.to_vec(),
            )
        };

        self.buffer_lengths.insert(buffer_id, buflen);
        self.buffer_types.insert(buffer_id, BufferType::Stereo);
        self.control_q_send
            .send(ControlMessage::LoadSample(
                buffer_id,
                buflen,
                SampleBuffer::Stereo(buffer_left, buffer_right),
            ))
            .unwrap();
        // return bufnum
        buffer_id
    }
}
