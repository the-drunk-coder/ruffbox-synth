use crate::building_blocks::{Modulator, MonoSource, SynthParameterLabel, SynthParameterValue};

/**
 * A non-band-limited square-wave oscillator.
 */
pub struct LFSquare<const BUFSIZE: usize> {
    //user values
    freq: f32,
    lvl: f32,
    pulsewidth: f32,

    // internal values
    samplerate: f32,
    period_samples: usize,
    period_count: usize,
    flank_point: usize,

    // modulator slots
    freq_mod: Option<Modulator<BUFSIZE>>, // currently allows modulating frequency ..
    lvl_mod: Option<Modulator<BUFSIZE>>,  // and level
    pw_mod: Option<Modulator<BUFSIZE>>,   // and level
}

impl<const BUFSIZE: usize> LFSquare<BUFSIZE> {
    pub fn new(freq: f32, pulsewidth: f32, lvl: f32, samplerate: f32) -> Self {
        LFSquare {
            freq,
            lvl,
            samplerate,
            pulsewidth,
            period_samples: (samplerate / freq).round() as usize,
            period_count: 0,
            flank_point: ((samplerate / freq).round() * pulsewidth) as usize,
            freq_mod: None,
            lvl_mod: None,
            pw_mod: None,
        }
    }
}

impl<const BUFSIZE: usize> MonoSource<BUFSIZE> for LFSquare<BUFSIZE> {
    // some parameter limits might be nice ...
    fn set_parameter(&mut self, par: SynthParameterLabel, value: &SynthParameterValue) {
        match par {
            SynthParameterLabel::PitchFrequency => match value {
                SynthParameterValue::ScalarF32(f) => {
                    self.freq = *f;
                    self.period_samples = (self.samplerate / *f).round() as usize;
                    self.flank_point =
                        (self.period_samples as f32 * self.pulsewidth).round() as usize;
                }
                SynthParameterValue::Lfo(init, freq, range, op) => {
                    self.freq = *init;
                    self.freq_mod = Some(Modulator::lfo(*op, *freq, *range, self.samplerate))
                }
                _ => {}
            },
            SynthParameterLabel::Level => match value {
                SynthParameterValue::ScalarF32(l) => {
                    self.lvl = *l;
                }
                SynthParameterValue::Lfo(init, freq, range, op) => {
                    self.lvl = *init;
                    self.lvl_mod = Some(Modulator::lfo(*op, *freq, *range, self.samplerate))
                }
                _ => {}
            },
            SynthParameterLabel::Pulsewidth => match value {
                SynthParameterValue::ScalarF32(pw) => {
                    self.pulsewidth = *pw;
                    self.flank_point = (self.period_samples as f32 * pw).round() as usize;
                }
                SynthParameterValue::Lfo(init, freq, range, op) => {
                    self.pulsewidth = *init;
                    self.pw_mod = Some(Modulator::lfo(*op, *freq, *range, self.samplerate))
                }
                _ => {}
            },
            _ => (),
        };
    }

    fn finish(&mut self) {}

    fn is_finished(&self) -> bool {
        false
    }

    fn get_next_block(&mut self, start_sample: usize, in_buffers: &[Vec<f32>]) -> [f32; BUFSIZE] {
        let mut out_buf: [f32; BUFSIZE] = [0.0; BUFSIZE];

        if self.freq_mod.is_some() || self.lvl_mod.is_some() {
            let lvl_buf = if let Some(m) = self.lvl_mod.as_mut() {
                m.process(self.lvl, start_sample, in_buffers)
            } else {
                [self.lvl; BUFSIZE]
            };

            let freq_buf = if let Some(m) = self.freq_mod.as_mut() {
                m.process(self.freq, start_sample, in_buffers)
            } else {
                [self.freq; BUFSIZE]
            };
            let pw_buf = if let Some(m) = self.pw_mod.as_mut() {
                m.process(self.pulsewidth, start_sample, in_buffers)
            } else {
                [self.pulsewidth; BUFSIZE]
            };

            for (idx, current_sample) in out_buf
                .iter_mut()
                .enumerate()
                .take(BUFSIZE)
                .skip(start_sample)
            {
                self.period_samples = (self.samplerate / freq_buf[idx]).round() as usize;
                self.flank_point = (self.period_samples as f32 * pw_buf[idx]).round() as usize;

                if self.period_count < self.flank_point {
                    *current_sample = lvl_buf[idx];
                } else {
                    *current_sample = -lvl_buf[idx];
                }

                self.period_count += 1;

                if self.period_count > self.period_samples {
                    self.period_count = 0;
                }
            }
        } else {
            for current_sample in out_buf.iter_mut().take(BUFSIZE).skip(start_sample) {
                if self.period_count < self.flank_point {
                    *current_sample = self.lvl;
                } else {
                    *current_sample = -self.lvl;
                }

                self.period_count += 1;

                if self.period_count > self.period_samples {
                    self.period_count = 0;
                }
            }
        }

        out_buf
    }
}
