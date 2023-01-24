use crate::building_blocks::{
    Modulator, MonoSource, SampleBuffer, SynthParameterLabel, SynthParameterValue,
};

/**
 * A non-band-limited cubic sine approximation oscillator.
 */
#[derive(Clone)]
pub struct LFCub<const BUFSIZE: usize> {
    // user parameters
    freq: f32,
    amp: f32,

    // internal parameters
    internal_freq: f32,
    phase: f32,
    sample_period: f32,

    // modulator slots
    freq_mod: Option<Modulator<BUFSIZE>>,
    amp_mod: Option<Modulator<BUFSIZE>>,
}

impl<const BUFSIZE: usize> LFCub<BUFSIZE> {
    pub fn new(freq: f32, amp: f32, samplerate: f32) -> Self {
        LFCub {
            freq,
            amp,
            internal_freq: freq * (2.0 / samplerate),
            sample_period: 2.0 / samplerate,
            phase: 0.0,
            freq_mod: None,
            amp_mod: None,
        }
    }
}

impl<const BUFSIZE: usize> MonoSource<BUFSIZE> for LFCub<BUFSIZE> {
    fn reset(&mut self) {}

    fn set_modulator(
        &mut self,
        par: SynthParameterLabel,
        init: f32,
        modulator: Modulator<BUFSIZE>,
    ) {
        match par {
            SynthParameterLabel::PitchFrequency => {
                self.freq = init;
                self.internal_freq = self.freq * self.sample_period;
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
            SynthParameterLabel::PitchFrequency => {
                if let SynthParameterValue::ScalarF32(f) = value {
                    self.freq = *f;
                    self.internal_freq = *f * self.sample_period;
                }
            }
            SynthParameterLabel::OscillatorAmplitude => {
                if let SynthParameterValue::ScalarF32(a) = value {
                    self.amp = *a;
                }
            }
            _ => (),
        };
    }

    fn finish(&mut self) {}

    fn is_finished(&self) -> bool {
        false
    }

    fn get_next_block(
        &mut self,
        start_sample: usize,
        in_buffers: &[SampleBuffer],
    ) -> [f32; BUFSIZE] {
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

            let mut z: f32;
            for (idx, current_sample) in out_buf
                .iter_mut()
                .enumerate()
                .take(BUFSIZE)
                .skip(start_sample)
            {
                self.internal_freq = freq_buf[idx] * self.sample_period;

                if self.phase < 1.0 {
                    z = self.phase;
                } else if self.phase < 2.0 {
                    z = 2.0 - self.phase;
                } else {
                    self.phase -= 2.0;
                    z = self.phase;
                }
                self.phase += self.internal_freq;
                *current_sample = amp_buf[idx] * z * z * (6.0 - 4.0 * z) - 1.0;
            }
        } else {
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
                self.phase += self.internal_freq;
                *current_sample = self.amp * z * z * (6.0 - 4.0 * z) - 1.0;
            }
        }

        out_buf
    }
}
