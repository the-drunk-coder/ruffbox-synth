use crate::building_blocks::{
    Modulator, MonoSource, SynthParameterLabel, SynthParameterValue, ValueOrModulator,
};

/**
 * A non-band-limited triangle oscillator.
 *
 * Probably the most naive implementation of such an oscillator,
 * doesn't tune well, etc ...
 */
#[derive(Clone)]
pub struct LFTri<const BUFSIZE: usize> {
    // user parameters
    freq: f32,
    amp: f32,

    // internal parameters
    samplerate: f32,
    amp_inc_dec: f32,
    cur_amp: f32,
    rise: bool,

    // modulator slots
    freq_mod: Option<Modulator<BUFSIZE>>, // allows modulating frequency ..
    amp_mod: Option<Modulator<BUFSIZE>>,  // and level
}

impl<const BUFSIZE: usize> LFTri<BUFSIZE> {
    pub fn new(freq: f32, amp: f32, samplerate: f32) -> Self {
        LFTri {
            freq,
            amp,
            samplerate,
            amp_inc_dec: -2.0 / (samplerate / freq),
            rise: true,
            cur_amp: 0.0,
            freq_mod: None,
            amp_mod: None,
        }
    }
}

impl<const BUFSIZE: usize> MonoSource<BUFSIZE> for LFTri<BUFSIZE> {
    fn set_modulator(
        &mut self,
        par: SynthParameterLabel,
        init: f32,
        modulator: Modulator<BUFSIZE>,
    ) {
        match par {
            SynthParameterLabel::PitchFrequency => {
                self.freq = init;
                self.amp_inc_dec = -2.0 / (self.samplerate / self.freq);
                self.freq_mod = Some(modulator);
            }
            SynthParameterLabel::OscillatorAmplitude => {
                self.amp = init;
                self.amp_mod = Some(modulator);
            }
            _ => {}
        }
    }
    // some parameter limits might be nice ...
    fn set_parameter(&mut self, par: SynthParameterLabel, value: &SynthParameterValue) {
        match par {
            SynthParameterLabel::OscillatorPhaseEffective => {
                if let SynthParameterValue::ScalarF32(p) = value {
                    self.cur_amp = ((p / self.amp) + 1.0) / 2.0;
                }
            }
            SynthParameterLabel::PitchFrequency => {
                if let SynthParameterValue::ScalarF32(f) = value {
                    self.freq = *f;
                    self.amp_inc_dec = -2.0 / (self.samplerate / self.freq);
                }
            }
            SynthParameterLabel::OscillatorAmplitude => {
                if let SynthParameterValue::ScalarF32(l) = value {
                    self.amp = *l;
                }
            }
            _ => (),
        };
    }

    fn finish(&mut self) {}

    fn is_finished(&self) -> bool {
        false
    }

    fn reset(&mut self) {}

    fn get_next_block(&mut self, start_sample: usize, in_buffers: &[Vec<f32>]) -> [f32; BUFSIZE] {
        let mut out_buf: [f32; BUFSIZE] = [0.0; BUFSIZE];

        if self.freq_mod.is_some() || self.amp_mod.is_some() {
            let amp_buf = if let Some(m) = self.amp_mod.as_mut() {
                m.process(self.amp, start_sample, in_buffers)
            } else {
                [self.amp; BUFSIZE]
            };

            let freq_buf = if let Some(m) = self.freq_mod.as_mut() {
                m.process(self.freq, start_sample, in_buffers)
            } else {
                [self.freq; BUFSIZE]
            };

            for (idx, current_sample) in out_buf
                .iter_mut()
                .enumerate()
                .take(BUFSIZE)
                .skip(start_sample)
            {
                self.rise = if self.cur_amp > 1.0 {
                    self.cur_amp = 1.0;
                    false
                } else if self.cur_amp < 0.0 {
                    self.cur_amp = 0.0;
                    true
                } else {
                    self.rise
                };

                *current_sample = (self.cur_amp * 2.0 - 1.0) * amp_buf[idx];

                self.amp_inc_dec = if self.rise {
                    2.0 / (self.samplerate / freq_buf[idx])
                } else {
                    -2.0 / (self.samplerate / freq_buf[idx])
                };

                self.cur_amp += self.amp_inc_dec;
            }
        } else {
            for current_sample in out_buf.iter_mut().take(BUFSIZE).skip(start_sample) {
                if self.cur_amp > 1.0 {
                    self.cur_amp = 1.0;
                    self.amp_inc_dec *= -1.0;
                } else if self.cur_amp < 0.0 {
                    self.cur_amp = 0.0;
                    self.amp_inc_dec *= -1.0;
                }

                *current_sample = (self.cur_amp * 2.0 - 1.0) * self.amp;
                self.cur_amp += self.amp_inc_dec;
            }
        }

        out_buf
    }
}
