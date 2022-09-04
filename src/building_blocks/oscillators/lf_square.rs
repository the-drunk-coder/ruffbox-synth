use crate::building_blocks::{
    Modulator, MonoSource, SynthParameterLabel, SynthParameterValue, ValueOrModulator,
};

/**
 * A non-band-limited square-wave oscillator.
 *
 * Probably the most naive implementation of such an oscillator,
 * doesn't tune well, etc ...
 */
#[derive(Clone)]
pub struct LFSquare<const BUFSIZE: usize> {
    //user values
    freq: f32,
    amp: f32,
    pulsewidth: f32,

    // internal values
    samplerate: f32,
    period_samples: usize,
    period_count: usize,
    flank_point: usize,

    // modulator slots
    freq_mod: Option<Modulator<BUFSIZE>>, // currently allows modulating frequency ..
    amp_mod: Option<Modulator<BUFSIZE>>,  // and level
    pw_mod: Option<Modulator<BUFSIZE>>,   // and pulsewidth
}

impl<const BUFSIZE: usize> LFSquare<BUFSIZE> {
    pub fn new(freq: f32, pulsewidth: f32, amp: f32, samplerate: f32) -> Self {
        LFSquare {
            freq,
            amp,
            samplerate,
            pulsewidth,
            period_samples: (samplerate / freq).round() as usize,
            period_count: 0,
            flank_point: ((samplerate / freq).round() * pulsewidth) as usize,
            freq_mod: None,
            amp_mod: None,
            pw_mod: None,
        }
    }
}

impl<const BUFSIZE: usize> MonoSource<BUFSIZE> for LFSquare<BUFSIZE> {
    fn reset(&mut self) {}

    fn set_param_or_modulator(
        &mut self,
        par: SynthParameterLabel,
        val_or_mod: ValueOrModulator<BUFSIZE>,
    ) {
        match val_or_mod {
            ValueOrModulator::Val(val) => self.set_parameter(par, &val),
            ValueOrModulator::Mod(init, modulator) => self.set_modulator(par, init, modulator),
        }
    }

    fn set_modulator(
        &mut self,
        par: SynthParameterLabel,
        init: f32,
        modulator: Modulator<BUFSIZE>,
    ) {
        match par {
            SynthParameterLabel::PitchFrequency => {
                self.freq = init;
                self.freq_mod = Some(modulator);
            }
            SynthParameterLabel::OscillatorAmplitude => {
                self.amp = init;
                self.amp_mod = Some(modulator);
            }
            SynthParameterLabel::Pulsewidth => {
                self.pulsewidth = init;
                self.pw_mod = Some(modulator);
            }
            _ => {}
        }
    }

    fn set_parameter(&mut self, par: SynthParameterLabel, value: &SynthParameterValue) {
        match par {
            SynthParameterLabel::PitchFrequency => {
                if let SynthParameterValue::ScalarF32(f) = value {
                    self.freq = *f;
                    self.period_samples = (self.samplerate / *f).round() as usize;
                    self.flank_point =
                        (self.period_samples as f32 * self.pulsewidth).round() as usize;
                }
            }
            SynthParameterLabel::OscillatorAmplitude => {
                if let SynthParameterValue::ScalarF32(l) = value {
                    self.amp = *l;
                }
            }
            SynthParameterLabel::Pulsewidth => {
                if let SynthParameterValue::ScalarF32(pw) = value {
                    self.pulsewidth = *pw;
                    self.flank_point = (self.period_samples as f32 * pw).round() as usize;
                }
            }
            _ => (),
        };
    }

    fn finish(&mut self) {}

    fn is_finished(&self) -> bool {
        false
    }

    fn get_next_block(&mut self, start_sample: usize, in_buffers: &[Vec<f32>]) -> [f32; BUFSIZE] {
        let mut out_buf: [f32; BUFSIZE] = [0.0; BUFSIZE];

        if self.freq_mod.is_some() || self.amp_mod.is_some() || self.pw_mod.is_some() {
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
                    *current_sample = amp_buf[idx];
                } else {
                    *current_sample = -amp_buf[idx];
                }

                self.period_count += 1;

                if self.period_count > self.period_samples {
                    self.period_count = 0;
                }
            }
        } else {
            for current_sample in out_buf.iter_mut().take(BUFSIZE).skip(start_sample) {
                if self.period_count < self.flank_point {
                    *current_sample = self.amp;
                } else {
                    *current_sample = -self.amp;
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
