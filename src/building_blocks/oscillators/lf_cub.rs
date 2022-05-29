use crate::building_blocks::{Modulator, MonoSource, SynthParameterLabel, SynthParameterValue};

/**
 * A non-band-limited cubic sine approximation oscillator.
 */
pub struct LFCub<const BUFSIZE: usize> {
    // user parameters
    freq: f32,
    lvl: f32,

    // internal parameters
    internal_freq: f32,
    samplerate: f32,
    phase: f32,
    sample_period: f32,

    // modulator slots
    freq_mod: Option<Modulator<BUFSIZE>>,
    lvl_mod: Option<Modulator<BUFSIZE>>,
}

impl<const BUFSIZE: usize> LFCub<BUFSIZE> {
    pub fn new(freq: f32, lvl: f32, samplerate: f32) -> Self {
        LFCub {
            freq,
            lvl,
            samplerate,
            internal_freq: freq * (2.0 / samplerate),
            sample_period: 2.0 / samplerate,
            phase: 0.0,
            freq_mod: None,
            lvl_mod: None,
        }
    }
}

impl<const BUFSIZE: usize> MonoSource<BUFSIZE> for LFCub<BUFSIZE> {
    // some parameter limits might be nice ...
    fn set_parameter(&mut self, par: SynthParameterLabel, value: &SynthParameterValue) {
        match par {
            SynthParameterLabel::PitchFrequency => match value {
                SynthParameterValue::ScalarF32(f) => {
                    self.freq = *f;
                    self.internal_freq = *f * self.sample_period;
                }
                SynthParameterValue::Lfo(init, freq, amp, add, op) => {
                    self.freq = *init;
                    self.freq_mod = Some(Modulator::lfo(
                        *op,
                        *freq,
                        *amp,
                        *add,
                        false,
                        false,
                        self.samplerate,
                    ))
                }
                SynthParameterValue::LFSaw(init, freq, amp, add, op) => {
                    self.freq = *init;
                    self.freq_mod = Some(Modulator::lfsaw(
                        *op,
                        *freq,
                        *amp,
                        *add,
                        false,
                        false,
                        self.samplerate,
                    ))
                }
                SynthParameterValue::LFTri(init, freq, amp, add, op) => {
                    self.freq = *init;
                    self.freq_mod = Some(Modulator::lftri(
                        *op,
                        *freq,
                        *amp,
                        *add,
                        false,
                        false,
                        self.samplerate,
                    ))
                }
                SynthParameterValue::LFSquare(init, freq, pw, amp, add, op) => {
                    self.freq = *init;
                    self.freq_mod = Some(Modulator::lfsquare(
                        *op,
                        *freq,
                        *pw,
                        *amp,
                        *add,
                        false,
                        false,
                        self.samplerate,
                    ))
                }
                _ => {}
            },
            SynthParameterLabel::OscillatorLevel => match value {
                SynthParameterValue::ScalarF32(l) => {
                    self.lvl = *l;
                }
                SynthParameterValue::Lfo(init, freq, amp, add, op) => {
                    self.lvl = *init;
                    self.lvl_mod = Some(Modulator::lfo(
                        *op,
                        *freq,
                        *amp,
                        *add,
                        false,
                        false,
                        self.samplerate,
                    ))
                }
                SynthParameterValue::LFTri(init, freq, amp, add, op) => {
                    self.lvl = *init;
                    self.lvl_mod = Some(Modulator::lftri(
                        *op,
                        *freq,
                        *amp,
                        *add,
                        false,
                        false,
                        self.samplerate,
                    ))
                }
                SynthParameterValue::LFSaw(init, freq, amp, add, op) => {
                    self.lvl = *init;
                    self.lvl_mod = Some(Modulator::lfsaw(
                        *op,
                        *freq,
                        *amp,
                        *add,
                        false,
                        false,
                        self.samplerate,
                    ))
                }
                SynthParameterValue::LFSquare(init, freq, pw, amp, add, op) => {
                    self.lvl = *init;
                    self.lvl_mod = Some(Modulator::lfsquare(
                        *op,
                        *freq,
                        *pw,
                        *amp,
                        *add,
                        false,
                        false,
                        self.samplerate,
                    ))
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
                *current_sample = lvl_buf[idx] * z * z * (6.0 - 4.0 * z) - 1.0;
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
                *current_sample = self.lvl * z * z * (6.0 - 4.0 * z) - 1.0;
            }
        }

        out_buf
    }
}
