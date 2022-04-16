use crate::building_blocks::{Modulator, MonoSource, SynthParameterLabel, SynthParameterValue};

use std::f32::consts::PI;

/**
 * A simple sine oscillator
 */
#[derive(Clone, Copy)]
pub struct SineOsc<const BUFSIZE: usize> {
    lvl: f32,
    sin_time: f32,
    sin_delta_time: f32,
    pi_slice: f32,
    sample_count: u64,
}

impl<const BUFSIZE: usize> SineOsc<BUFSIZE> {
    pub fn new(freq: f32, lvl: f32, sr: f32) -> Self {
        SineOsc {
            lvl,
            sin_time: 0.0,
            sin_delta_time: 1.0 / sr,
            pi_slice: 2.0 * PI * freq,
            sample_count: 0,
        }
    }
}

impl<const BUFSIZE: usize> MonoSource<BUFSIZE> for SineOsc<BUFSIZE> {
    // some parameter limits might be nice ...
    fn set_parameter(&mut self, par: SynthParameterLabel, value: &SynthParameterValue) {
        match par {
            SynthParameterLabel::PitchFrequency => {
                if let SynthParameterValue::ScalarF32(f) = value {
                    self.pi_slice = 2.0 * PI * f;
                };
            }
            SynthParameterLabel::Level => {
                if let SynthParameterValue::ScalarF32(l) = value {
                    self.lvl = *l
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

    fn get_next_block(
        &mut self,
        start_sample: usize,
        _: &[Vec<f32>],
        _: &[Modulator<BUFSIZE>],
    ) -> [f32; BUFSIZE] {
        let mut out_buf: [f32; BUFSIZE] = [0.0; BUFSIZE];

        for current_sample in out_buf.iter_mut().take(BUFSIZE).skip(start_sample) {
            *current_sample =
                (self.pi_slice * self.sin_delta_time * self.sample_count as f32).sin() * self.lvl;
            self.sample_count += 1;
            self.sin_time += self.sin_delta_time;
        }

        out_buf
    }
}
