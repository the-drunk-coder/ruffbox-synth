use crate::building_blocks::{Modulator, MonoSource, SynthParameterLabel, SynthParameterValue};

/**
 * A brown noise generator based on wyrand (through fastrand)
 */
#[derive(Clone)]
pub struct BrownNoise<const BUFSIZE: usize> {
    step: f32,
    amp: f32,
    cur: f32,
    amp_mod: Option<Modulator<BUFSIZE>>, // and level
}

impl<const BUFSIZE: usize> BrownNoise<BUFSIZE> {
    pub fn new(amp: f32, step: f32) -> Self {
        BrownNoise {
            step,
            amp,
            cur: 0.0,
            amp_mod: None,
        }
    }
}

impl<const BUFSIZE: usize> MonoSource<BUFSIZE> for BrownNoise<BUFSIZE> {
    fn reset(&mut self) {}

    fn set_modulator(
        &mut self,
        par: SynthParameterLabel,
        init: f32,
        modulator: Modulator<BUFSIZE>,
    ) {
        if par == SynthParameterLabel::OscillatorAmplitude {
            self.amp = init;
            self.amp_mod = Some(modulator);
        }
    }

    fn set_parameter(&mut self, par: SynthParameterLabel, value: &SynthParameterValue) {
        if par == SynthParameterLabel::OscillatorAmplitude {
            if let SynthParameterValue::ScalarF32(l) = value {
                self.amp = *l;
            }
        }
    }

    fn finish(&mut self) {}

    fn is_finished(&self) -> bool {
        false
    }

    fn get_next_block(&mut self, start_sample: usize, in_buffers: &[Vec<f32>]) -> [f32; BUFSIZE] {
        let mut out_buf: [f32; BUFSIZE] = [0.0; BUFSIZE];

        if let Some(m) = self.amp_mod.as_mut() {
            let amp_buf = m.process(self.amp, start_sample, in_buffers);

            for (idx, current_sample) in out_buf
                .iter_mut()
                .enumerate()
                .take(BUFSIZE)
                .skip(start_sample)
            {
                if fastrand::bool() {
                    self.cur += self.step;
                } else {
                    self.cur -= self.step;
                }

                if self.cur >= 1.0 {
                    self.cur -= 2.0;
                } else if self.cur <= -1.0 {
                    self.cur += 2.0;
                }

                *current_sample = self.cur * amp_buf[idx];
            }
        } else {
            for current_sample in out_buf.iter_mut().take(BUFSIZE).skip(start_sample) {
                if fastrand::bool() {
                    self.cur += self.step;
                } else {
                    self.cur -= self.step;
                }

                if self.cur >= 1.0 {
                    self.cur -= 2.0;
                } else if self.cur <= -1.0 {
                    self.cur += 2.0;
                }

                *current_sample = self.cur * self.amp;
            }
        }

        out_buf
    }
}
