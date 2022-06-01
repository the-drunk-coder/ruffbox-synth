use crate::building_blocks::{Modulator, MonoSource, SynthParameterLabel, SynthParameterValue};

/**
 * A non-band-limited sawtooth oscillator.
 */
pub struct LFSaw<const BUFSIZE: usize> {
    // user parameters
    freq: f32,
    amp: f32,

    // internal parameters
    samplerate: f32,
    amp_inc: f32,
    cur_amp: f32,

    // modulator slots
    freq_mod: Option<Modulator<BUFSIZE>>, // allows modulating frequency ..
    amp_mod: Option<Modulator<BUFSIZE>>,  // and level
}

impl<const BUFSIZE: usize> LFSaw<BUFSIZE> {
    pub fn new(freq: f32, amp: f32, samplerate: f32) -> Self {
        LFSaw {
            freq,
            amp,
            samplerate,            
            amp_inc: (2.0 * amp) / (samplerate / freq),
            cur_amp: -1.0 * amp,
            freq_mod: None,
            amp_mod: None,
        }
    }
}

impl<const BUFSIZE: usize> MonoSource<BUFSIZE> for LFSaw<BUFSIZE> {
    // some parameter limits might be nice ...
    fn set_parameter(&mut self, par: SynthParameterLabel, value: &SynthParameterValue) {
        match par {
            SynthParameterLabel::PitchFrequency => match value {
                SynthParameterValue::ScalarF32(f) => {
                    self.freq = *f;                    
                    self.amp_inc = (2.0 * self.amp) / (self.samplerate / f);
                }
                SynthParameterValue::Lfo(init, freq, eff_phase, amp, add, op) => {
                    self.freq = *init;
                    self.freq_mod = Some(Modulator::lfo(
                        *op,
                        *freq,
                        *eff_phase,
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
            SynthParameterLabel::OscillatorAmplitude => match value {
                SynthParameterValue::ScalarF32(l) => {
                    self.amp = *l;
                    self.amp_inc = (2.0 * self.amp) / (self.samplerate / self.freq);
                }
                SynthParameterValue::Lfo(init, freq, eff_phase, amp, add, op) => {
                    self.amp = *init;
                    self.amp_mod = Some(Modulator::lfo(
                        *op,
                        *freq,
                        *eff_phase,
                        *amp,
                        *add,
                        false,
                        false,
                        self.samplerate,
                    ))
                }
                SynthParameterValue::LFTri(init, freq, amp, add, op) => {
                    self.amp = *init;
                    self.amp_mod = Some(Modulator::lftri(
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
                    self.amp = *init;
                    self.amp_mod = Some(Modulator::lfsaw(
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
                    self.amp = *init;
                    self.amp_mod = Some(Modulator::lfsquare(
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
                if self.cur_amp >= self.amp {
                    self.cur_amp = -1.0 * self.amp;
                    *current_sample = self.amp;
                } else if self.cur_amp <= -self.amp {
                    *current_sample = -self.amp;
                } else {
                    *current_sample = self.cur_amp;
                }

                self.amp_inc = (2.0 * amp_buf[idx]) / (self.samplerate / freq_buf[idx]);
                self.cur_amp += self.amp_inc;
            }
        } else {
            for current_sample in out_buf.iter_mut().take(BUFSIZE).skip(start_sample) {
                if self.cur_amp >= self.amp {
                    self.cur_amp = -1.0 * self.amp;
                    *current_sample = self.amp;
                } else if self.cur_amp <= -self.amp {
                    *current_sample = -self.amp;
                } else {
                    *current_sample = self.cur_amp;
                }

                self.cur_amp += self.amp_inc;
            }
        }

        out_buf
    }
}
