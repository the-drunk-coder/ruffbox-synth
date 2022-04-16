use crate::building_blocks::{Modulator, MonoSource, SynthParameterLabel, SynthParameterValue};

/**
 * A non-band-limited cubic sine approximation oscillator.
 */
pub struct LFCub<const BUFSIZE: usize> {
    lvl: f32,
    samplerate: f32,
    freq: f32,
    phase: f32,
}

impl<const BUFSIZE: usize> LFCub<BUFSIZE> {
    pub fn new(freq: f32, lvl: f32, samplerate: f32) -> Self {
        LFCub {
            //freq: freq,
            lvl,
            samplerate,
            phase: 0.0,
            freq,
        }
    }
}

impl<const BUFSIZE: usize> MonoSource<BUFSIZE> for LFCub<BUFSIZE> {
    // some parameter limits might be nice ...
    fn set_parameter(&mut self, par: SynthParameterLabel, value: &SynthParameterValue) {
        match par {
            SynthParameterLabel::PitchFrequency => {
                if let SynthParameterValue::ScalarF32(f) = value {
                    self.freq = *f * (1.0 / self.samplerate);
                }
            }
            SynthParameterLabel::Level => {
                if let SynthParameterValue::ScalarF32(l) = value {
                    self.lvl = *l;
                }
            }
            _ => (),
        };
    }

    fn finish(&mut self) {}

    fn is_finished(&self) -> bool {
        false
    }

    fn get_next_block(&mut self, start_sample: usize, _: &[Vec<f32>]) -> [f32; BUFSIZE] {
        let mut out_buf: [f32; BUFSIZE] = [0.0; BUFSIZE];

        let mut z: f32;
        for current_sample in out_buf.iter_mut().take(BUFSIZE).skip(start_sample) {
            if self.phase < 1.0 {
                z = self.phase;
            } else if self.phase < 2.0 {
                z = 2.0 - self.phase;
            } else {
                self.phase -= 2.0;
                z = self.phase;
            }
            self.phase += self.freq;
            *current_sample = self.lvl * z * z * (6.0 - 4.0 * z) - 1.0;
        }

        out_buf
    }
}
