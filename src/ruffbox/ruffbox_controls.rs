use rubato::{FftFixedIn, Resampler};

// crossbeam for the event queue
use crossbeam::atomic::AtomicCell;

use crate::ruffbox::synth::synths::*;
use crate::ruffbox::synth::SourceType;
use crate::ruffbox::synth::SynthParameter;
use crate::ruffbox::ControlMessage;
use crate::ruffbox::ScheduledEvent;
use dashmap::DashMap;
use std::sync::Arc;

/// thin wrapper around a scheduled event instasnce
pub struct PreparedInstance<const BUFSIZE: usize, const NCHAN: usize> {
    ev: ScheduledEvent<BUFSIZE, NCHAN>,
}

impl<const BUFSIZE: usize, const NCHAN: usize> PreparedInstance<BUFSIZE, NCHAN> {
    pub fn set_instance_parameter(&mut self, par: SynthParameter, val: f32) {
        self.ev.set_parameter(par, val);
    }
}

/// These are the controls, the part which you use in your control thread
/// to control the Ruffbox, trigger playback, etc ...
pub struct RuffboxControls<const BUFSIZE: usize, const NCHAN: usize> {
    // Buffer lengths need to be known when initializing sampler instances,
    // which is why unfortunately we need to mirror them here in the controls.
    // Thanks to the magic of DashMap, we can get around having to
    // use &mut self. Maybe one day I'll find out how to make the controls
    // actually stateless, but until then, the interior mutability pattern
    // comes in handy ...
    buffer_counter: AtomicCell<usize>,
    buffer_lengths: DashMap<usize, usize>,
    max_buffers: usize,
    control_q_send: crossbeam::channel::Sender<ControlMessage<BUFSIZE, NCHAN>>,
    now: Arc<AtomicCell<f64>>, // shared reference to global time counter
    pub samplerate: f32,       // finally after all those years ...
}

impl<const BUFSIZE: usize, const NCHAN: usize> RuffboxControls<BUFSIZE, NCHAN> {
    pub(crate) fn new(
        samplerate: f64,
        life_buffer: bool,
        max_buffers: usize,
        freeze_buffers: usize,
        now: &Arc<AtomicCell<f64>>,
        tx: crossbeam::channel::Sender<ControlMessage<BUFSIZE, NCHAN>>,
    ) -> RuffboxControls<BUFSIZE, NCHAN> {
        RuffboxControls {
            buffer_counter: AtomicCell::new(if life_buffer { 1 + freeze_buffers } else { 0 }),
            buffer_lengths: DashMap::new(),
            max_buffers,
            control_q_send: tx,
            samplerate: samplerate as f32,
            now: Arc::clone(now),
        }
    }

    /// prepare a sound source instance, return instance id
    pub fn prepare_instance(
        &self,
        src_type: SourceType,
        timestamp: f64,
        sample_buf: usize,
    ) -> PreparedInstance<BUFSIZE, NCHAN> {
        PreparedInstance {
            ev: match src_type {
                SourceType::SineOsc => {
                    ScheduledEvent::new(timestamp, Box::new(SineSynth::new(self.samplerate)))
                }
                SourceType::SineSynth => {
                    ScheduledEvent::new(timestamp, Box::new(SineSynth::new(self.samplerate)))
                }
                SourceType::LFTriangleSynth => {
                    ScheduledEvent::new(timestamp, Box::new(LFTriSynth::new(self.samplerate)))
                }
                SourceType::RissetBell => {
                    ScheduledEvent::new(timestamp, Box::new(RissetBell::new(self.samplerate)))
                }
                SourceType::Sampler => ScheduledEvent::new(
                    timestamp,
                    Box::new(NChannelSampler::with_bufnum_len(
                        sample_buf,
                        *self.buffer_lengths.get(&sample_buf).unwrap(),
                        self.samplerate,
                    )),
                ),
                SourceType::LiveSampler => ScheduledEvent::new(
                    timestamp,
                    Box::new(NChannelSampler::with_bufnum_len(
                        0,
                        *self.buffer_lengths.get(&0).unwrap(),
                        self.samplerate,
                    )),
                ),
                SourceType::LFSawSynth => {
                    ScheduledEvent::new(timestamp, Box::new(LFSawSynth::new(self.samplerate)))
                }
                SourceType::LFSquareSynth => {
                    ScheduledEvent::new(timestamp, Box::new(LFSquareSynth::new(self.samplerate)))
                }
                SourceType::LFCubSynth => {
                    ScheduledEvent::new(timestamp, Box::new(LFCubSynth::new(self.samplerate)))
                }
            },
        }
    }

    pub fn set_master_parameter(&self, par: SynthParameter, val: f32) {
        self.control_q_send
            .send(ControlMessage::SetGlobalParam(par, val))
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
    pub fn freeze_buffer(&self, freezbuf: usize) {
        self.control_q_send
            .send(ControlMessage::FreezeBuffer(freezbuf))
            .unwrap();
    }

    /// Loads a mono sample and returns the assigned buffer number.
    ///
    /// Resample to current samplerate if necessary and specified.
    /// The sample buffer is passed as mutable because the method adds
    /// interpolation samples without the need of a copy.
    pub fn load_sample(&self, samples: &mut Vec<f32>, resample: bool, sr: f32) -> usize {
        let buffer_id = self.buffer_counter.fetch_add(1);

        if buffer_id > self.max_buffers {
            println!("warning, this buffer won't be loaded !");
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
        self.control_q_send
            .send(ControlMessage::LoadSample(buffer_id, buflen, buffer))
            .unwrap();
        // return bufnum
        buffer_id
    }
}
