use std::sync::Arc;

use rubato::{FftFixedIn, Resampler};

// crossbeam for the event queue
use crossbeam::atomic::AtomicCell;
use dashmap::DashMap;

use crate::building_blocks::{Modulator, SynthParameterLabel, SynthParameterValue};
use crate::ruffbox::{ControlMessage, ScheduledEvent};
use crate::synths::*;

/// thin wrapper around a scheduled event instasnce
pub struct PreparedInstance<const BUFSIZE: usize, const NCHAN: usize> {
    ev: ScheduledEvent<BUFSIZE, NCHAN>,
    sr: f32,
}

impl<const BUFSIZE: usize, const NCHAN: usize> PreparedInstance<BUFSIZE, NCHAN> {
    pub fn set_instance_parameter(&mut self, par: SynthParameterLabel, val: &SynthParameterValue) {
        match val {
            SynthParameterValue::Lfo(init, freq, eff_phase, amp, add, op) => {
                self.ev.set_modulator(
                    par,
                    *init,
                    match par {
                        SynthParameterLabel::LowpassCutoffFrequency => {
                            Modulator::lfo(*op, *freq, *eff_phase, *amp, *add, true, false, self.sr)
                        }
                        SynthParameterLabel::HighpassCutoffFrequency => {
                            Modulator::lfo(*op, *freq, *eff_phase, *amp, *add, true, false, self.sr)
                        }
                        SynthParameterLabel::PeakFrequency => {
                            Modulator::lfo(*op, *freq, *eff_phase, *amp, *add, true, false, self.sr)
                        }
                        _ => Modulator::lfo(
                            *op, *freq, *eff_phase, *amp, *add, false, false, self.sr,
                        ),
                    },
                );
            }
            SynthParameterValue::LFSaw(init, freq, eff_phase, amp, add, op) => {
                self.ev.set_modulator(
                    par,
                    *init,
                    match par {
                        SynthParameterLabel::LowpassCutoffFrequency => Modulator::lfsaw(
                            *op, *freq, *eff_phase, *amp, *add, true, false, self.sr,
                        ),
                        SynthParameterLabel::HighpassCutoffFrequency => Modulator::lfsaw(
                            *op, *freq, *eff_phase, *amp, *add, true, false, self.sr,
                        ),
                        SynthParameterLabel::PeakFrequency => Modulator::lfsaw(
                            *op, *freq, *eff_phase, *amp, *add, true, false, self.sr,
                        ),
                        _ => Modulator::lfsaw(
                            *op, *freq, *eff_phase, *amp, *add, false, false, self.sr,
                        ),
                    },
                );
            }
            SynthParameterValue::LFRSaw(init, freq, eff_phase, amp, add, op) => {
                self.ev.set_modulator(
                    par,
                    *init,
                    match par {
                        SynthParameterLabel::LowpassCutoffFrequency => Modulator::lfrsaw(
                            *op, *freq, *eff_phase, *amp, *add, true, false, self.sr,
                        ),
                        SynthParameterLabel::HighpassCutoffFrequency => Modulator::lfrsaw(
                            *op, *freq, *eff_phase, *amp, *add, true, false, self.sr,
                        ),
                        SynthParameterLabel::PeakFrequency => Modulator::lfrsaw(
                            *op, *freq, *eff_phase, *amp, *add, true, false, self.sr,
                        ),
                        _ => Modulator::lfrsaw(
                            *op, *freq, *eff_phase, *amp, *add, false, false, self.sr,
                        ),
                    },
                );
            }
            SynthParameterValue::LFTri(init, freq, eff_phase, amp, add, op) => {
                self.ev.set_modulator(
                    par,
                    *init,
                    match par {
                        SynthParameterLabel::LowpassCutoffFrequency => Modulator::lftri(
                            *op, *freq, *eff_phase, *amp, *add, true, false, self.sr,
                        ),
                        SynthParameterLabel::HighpassCutoffFrequency => Modulator::lftri(
                            *op, *freq, *eff_phase, *amp, *add, true, false, self.sr,
                        ),
                        SynthParameterLabel::PeakFrequency => Modulator::lftri(
                            *op, *freq, *eff_phase, *amp, *add, true, false, self.sr,
                        ),
                        _ => Modulator::lftri(
                            *op, *freq, *eff_phase, *amp, *add, false, false, self.sr,
                        ),
                    },
                );
            }
            SynthParameterValue::LFSquare(init, freq, pw, amp, add, op) => {
                self.ev.set_modulator(
                    par,
                    *init,
                    match par {
                        SynthParameterLabel::LowpassCutoffFrequency => {
                            Modulator::lfsquare(*op, *freq, *pw, *amp, *add, true, false, self.sr)
                        }
                        SynthParameterLabel::HighpassCutoffFrequency => {
                            Modulator::lfsquare(*op, *freq, *pw, *amp, *add, true, false, self.sr)
                        }
                        SynthParameterLabel::PeakFrequency => {
                            Modulator::lfsquare(*op, *freq, *pw, *amp, *add, true, false, self.sr)
                        }
                        _ => {
                            Modulator::lfsquare(*op, *freq, *pw, *amp, *add, false, false, self.sr)
                        }
                    },
                );
            }

            SynthParameterValue::LinRamp(from, to, time, op) => {
                self.ev.set_modulator(
                    par,
                    *from,
                    Modulator::lin_ramp(*op, *from, *to, *time, self.sr),
                );
            }
            SynthParameterValue::LogRamp(from, to, time, op) => {
                self.ev.set_modulator(
                    par,
                    *from,
                    Modulator::log_ramp(*op, *from, *to, *time, self.sr),
                );
            }
            SynthParameterValue::ExpRamp(from, to, time, op) => {
                self.ev.set_modulator(
                    par,
                    *from,
                    Modulator::exp_ramp(*op, *from, *to, *time, self.sr),
                );
            }
            SynthParameterValue::MultiPointEnvelope(segments, loop_env, op) => {
                let init = if let Some(seg) = segments.first() {
                    seg.from
                } else {
                    0.0
                };
                self.ev.set_modulator(
                    par,
                    init,
                    Modulator::multi_point_envelope(*op, segments.to_vec(), *loop_env, self.sr),
                );
            }
            _ => {
                self.ev.set_parameter(par, val);
            }
        }
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
        if live_buffers > 0 {
            // create buffer lenghts for live buffers and freeze buffers
            for b in 0..live_buffers + freeze_buffers {
                buffer_lengths.insert(b, (samplerate * live_buffer_time) as usize);
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
                SynthType::SineOsc => {
                    ScheduledEvent::new(timestamp, Box::new(SineSynth::new(self.samplerate)))
                }
                SynthType::SineSynth => {
                    ScheduledEvent::new(timestamp, Box::new(SineSynth::new(self.samplerate)))
                }
                SynthType::LFTriangleSynth => {
                    ScheduledEvent::new(timestamp, Box::new(LFTriSynth::new(self.samplerate)))
                }
                SynthType::RissetBell => {
                    ScheduledEvent::new(timestamp, Box::new(RissetBell::new(self.samplerate)))
                }
                SynthType::Sampler => ScheduledEvent::new(
                    timestamp,
                    Box::new(NChannelSampler::with_bufnum_len(
                        sample_buf,
                        *self.buffer_lengths.get(&sample_buf).unwrap(),
                        self.samplerate,
                    )),
                ),
                SynthType::LiveSampler if self.num_live_buffers > 0 => {
                    let final_bufnum = if sample_buf < self.num_live_buffers {
                        sample_buf
                    } else {
                        0
                    };
                    ScheduledEvent::new(
                        timestamp,
                        Box::new(NChannelSampler::with_bufnum_len(
                            final_bufnum,
                            *self.buffer_lengths.get(&final_bufnum).unwrap(),
                            self.samplerate,
                        )),
                    )
                }
                SynthType::FrozenSampler if self.num_freeze_buffers > 0 => {
                    let final_bufnum = if sample_buf < self.num_freeze_buffers {
                        sample_buf + self.freeze_buffer_offset
                    } else {
                        self.freeze_buffer_offset
                    };
                    ScheduledEvent::new(
                        timestamp,
                        Box::new(NChannelSampler::with_bufnum_len(
                            final_bufnum,
                            *self.buffer_lengths.get(&final_bufnum).unwrap(),
                            self.samplerate,
                        )),
                    )
                }
                SynthType::LFSawSynth => {
                    ScheduledEvent::new(timestamp, Box::new(LFSawSynth::new(self.samplerate)))
                }
                SynthType::LFSquareSynth => {
                    ScheduledEvent::new(timestamp, Box::new(LFSquareSynth::new(self.samplerate)))
                }
                SynthType::LFCubSynth => {
                    ScheduledEvent::new(timestamp, Box::new(LFCubSynth::new(self.samplerate)))
                }
                SynthType::Wavetable => {
                    ScheduledEvent::new(timestamp, Box::new(WavetableSynth::new(self.samplerate)))
                }
                SynthType::Wavematrix => {
                    ScheduledEvent::new(timestamp, Box::new(WavematrixSynth::new(self.samplerate)))
                }
                _ => {
                    return None;
                } // jump out
            },
        })
    }

    pub fn set_master_parameter(&self, par: SynthParameterLabel, val: SynthParameterValue) {
        match val {
            SynthParameterValue::Lfo(init, freq, eff_phase, amp, add, op) => {
                self.control_q_send
                    .send(ControlMessage::SetGlobalModulator(
                        par,
                        init,
                        match par {
                            SynthParameterLabel::LowpassCutoffFrequency => Modulator::lfo(
                                op,
                                freq,
                                eff_phase,
                                amp,
                                add,
                                true,
                                false,
                                self.samplerate,
                            ),
                            SynthParameterLabel::HighpassCutoffFrequency => Modulator::lfo(
                                op,
                                freq,
                                eff_phase,
                                amp,
                                add,
                                true,
                                false,
                                self.samplerate,
                            ),
                            SynthParameterLabel::PeakFrequency => Modulator::lfo(
                                op,
                                freq,
                                eff_phase,
                                amp,
                                add,
                                true,
                                false,
                                self.samplerate,
                            ),
                            _ => Modulator::lfo(
                                op,
                                freq,
                                eff_phase,
                                amp,
                                add,
                                false,
                                false,
                                self.samplerate,
                            ),
                        },
                    ))
                    .unwrap();
            }
            SynthParameterValue::LFSaw(init, freq, eff_phase, amp, add, op) => {
                self.control_q_send
                    .send(ControlMessage::SetGlobalModulator(
                        par,
                        init,
                        match par {
                            SynthParameterLabel::LowpassCutoffFrequency => Modulator::lfsaw(
                                op,
                                freq,
                                eff_phase,
                                amp,
                                add,
                                true,
                                false,
                                self.samplerate,
                            ),
                            SynthParameterLabel::HighpassCutoffFrequency => Modulator::lfsaw(
                                op,
                                freq,
                                eff_phase,
                                amp,
                                add,
                                true,
                                false,
                                self.samplerate,
                            ),
                            SynthParameterLabel::PeakFrequency => Modulator::lfsaw(
                                op,
                                freq,
                                eff_phase,
                                amp,
                                add,
                                true,
                                false,
                                self.samplerate,
                            ),
                            _ => Modulator::lfsaw(
                                op,
                                freq,
                                eff_phase,
                                amp,
                                add,
                                false,
                                false,
                                self.samplerate,
                            ),
                        },
                    ))
                    .unwrap();
            }
            SynthParameterValue::LFRSaw(init, freq, eff_phase, amp, add, op) => {
                self.control_q_send
                    .send(ControlMessage::SetGlobalModulator(
                        par,
                        init,
                        match par {
                            SynthParameterLabel::LowpassCutoffFrequency => Modulator::lfrsaw(
                                op,
                                freq,
                                eff_phase,
                                amp,
                                add,
                                true,
                                false,
                                self.samplerate,
                            ),
                            SynthParameterLabel::HighpassCutoffFrequency => Modulator::lfrsaw(
                                op,
                                freq,
                                eff_phase,
                                amp,
                                add,
                                true,
                                false,
                                self.samplerate,
                            ),
                            SynthParameterLabel::PeakFrequency => Modulator::lfrsaw(
                                op,
                                freq,
                                eff_phase,
                                amp,
                                add,
                                true,
                                false,
                                self.samplerate,
                            ),
                            _ => Modulator::lfrsaw(
                                op,
                                freq,
                                eff_phase,
                                amp,
                                add,
                                false,
                                false,
                                self.samplerate,
                            ),
                        },
                    ))
                    .unwrap();
            }
            SynthParameterValue::LFTri(init, freq, eff_phase, amp, add, op) => {
                self.control_q_send
                    .send(ControlMessage::SetGlobalModulator(
                        par,
                        init,
                        match par {
                            SynthParameterLabel::LowpassCutoffFrequency => Modulator::lftri(
                                op,
                                freq,
                                eff_phase,
                                amp,
                                add,
                                true,
                                false,
                                self.samplerate,
                            ),
                            SynthParameterLabel::HighpassCutoffFrequency => Modulator::lftri(
                                op,
                                freq,
                                eff_phase,
                                amp,
                                add,
                                true,
                                false,
                                self.samplerate,
                            ),
                            SynthParameterLabel::PeakFrequency => Modulator::lftri(
                                op,
                                freq,
                                eff_phase,
                                amp,
                                add,
                                true,
                                false,
                                self.samplerate,
                            ),
                            _ => Modulator::lftri(
                                op,
                                freq,
                                eff_phase,
                                amp,
                                add,
                                false,
                                false,
                                self.samplerate,
                            ),
                        },
                    ))
                    .unwrap();
            }
            SynthParameterValue::LFSquare(init, freq, pw, amp, add, op) => {
                self.control_q_send
                    .send(ControlMessage::SetGlobalModulator(
                        par,
                        init,
                        match par {
                            SynthParameterLabel::LowpassCutoffFrequency => Modulator::lfsquare(
                                op,
                                freq,
                                pw,
                                amp,
                                add,
                                true,
                                false,
                                self.samplerate,
                            ),
                            SynthParameterLabel::HighpassCutoffFrequency => Modulator::lfsquare(
                                op,
                                freq,
                                pw,
                                amp,
                                add,
                                true,
                                false,
                                self.samplerate,
                            ),
                            SynthParameterLabel::PeakFrequency => Modulator::lfsquare(
                                op,
                                freq,
                                pw,
                                amp,
                                add,
                                true,
                                false,
                                self.samplerate,
                            ),
                            _ => Modulator::lfsquare(
                                op,
                                freq,
                                pw,
                                amp,
                                add,
                                false,
                                false,
                                self.samplerate,
                            ),
                        },
                    ))
                    .unwrap();
            }
            _ => {
                self.control_q_send
                    .send(ControlMessage::SetGlobalParam(par, val))
                    .unwrap();
            }
        }
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
