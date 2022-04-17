use crate::building_blocks::{Modulator, MonoSource, SynthParameterLabel, SynthParameterValue};

use std::f32::consts::PI;

/**
 * A simple sine oscillator
 */
pub struct SineOsc<const BUFSIZE: usize> {
    lvl: f32,
    sin_time: f32,
    sin_delta_time: f32,
    samplerate: f32,
    freq: f32,
    pi_slice: f32,

    freq_mod: Option<Modulator<BUFSIZE>>,
    lvl_mod: Option<Modulator<BUFSIZE>>,
}

impl<const BUFSIZE: usize> SineOsc<BUFSIZE> {
    pub fn new(freq: f32, lvl: f32, sr: f32) -> Self {
        SineOsc {
            lvl,
            sin_time: 0.0,
            sin_delta_time: 1.0 / sr,
            samplerate: sr,
            freq,
            pi_slice: 2.0 * PI * freq,

            freq_mod: None,
            lvl_mod: None,
        }
    }

    fn set_freq(&mut self, f: f32) {
        self.pi_slice = 2.0 * PI * f;
    }

    fn set_lvl(&mut self, l: f32) {
        self.lvl = l;
    }
}

impl<const BUFSIZE: usize> MonoSource<BUFSIZE> for SineOsc<BUFSIZE> {
    // some parameter limits might be nice ...
    fn set_parameter(&mut self, par: SynthParameterLabel, value: &SynthParameterValue) {
        match par {
            SynthParameterLabel::PitchFrequency => {
                match value {
                    SynthParameterValue::ScalarF32(f) => {
                        self.set_freq(*f);
                        self.freq = *f;
                    }
                    SynthParameterValue::Lfo(freq, range, op) => {
                        self.freq_mod = Some(Modulator::lfo(*op, *freq, *range, self.samplerate))
                    }
                    _ => { /* nothing to do, don't know how to handle this ... */ }
                }
            }
            SynthParameterLabel::Level => {
                match value {
                    SynthParameterValue::ScalarF32(l) => {
                        self.set_lvl(*l);
                    }
                    SynthParameterValue::Lfo(freq, range, op) => {
                        self.lvl_mod = Some(Modulator::lfo(*op, *freq, *range, self.samplerate))
                    }
                    _ => { /* nothing to do, don't know how to handle this ... */ }
                }
            }
            _ => (),
        };
    }

    fn finish(&mut self) {
        //self.state = SynthState::Finished;
    }

    fn is_finished(&self) -> bool {
        false
    }

    fn get_next_block(&mut self, start_sample: usize, _in_buffers: &[Vec<f32>]) -> [f32; BUFSIZE] {
        let mut out_buf: [f32; BUFSIZE] = [0.0; BUFSIZE];

        for current_sample in out_buf.iter_mut().take(BUFSIZE).skip(start_sample) {
            *current_sample = (self.pi_slice * self.sin_time as f32).sin() * self.lvl;
            self.sin_time += self.sin_delta_time;
        }

        out_buf
    }
}
