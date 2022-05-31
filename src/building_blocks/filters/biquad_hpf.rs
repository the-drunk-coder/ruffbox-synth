use crate::building_blocks::{Modulator, MonoEffect, SynthParameterLabel, SynthParameterValue};

/**
 * Biquad HiPass Filter, 12dB/oct
 */
pub struct BiquadHpf<const BUFSIZE: usize> {
    // user parameters
    cutoff: f32,
    q: f32,

    // internal parameters
    a1: f32,
    a2: f32,
    b0: f32,
    b1: f32,
    b2: f32,
    del1: f32,
    del2: f32,
    k: f32,
    samplerate: f32,

    // modulator slots
    cutoff_mod: Option<Modulator<BUFSIZE>>,
    q_mod: Option<Modulator<BUFSIZE>>,
}

impl<const BUFSIZE: usize> BiquadHpf<BUFSIZE> {
    pub fn new(freq: f32, q: f32, sr: f32) -> Self {
        let k = ((std::f32::consts::PI * freq) / sr).tanh();
        let k_pow_two = k.powf(2.0);
        let b0 = q / ((k_pow_two * q) + k + q);
        BiquadHpf {
            cutoff: freq,
            q,
            a1: (2.0 * q * (k_pow_two - 1.0)) / ((k_pow_two * q) + k + q),
            a2: ((k_pow_two * q) - k + q) / ((k_pow_two * q) + k + q),
            b0,
            b1: -1.0 * ((2.0 * q) / ((k_pow_two * q) + k + q)),
            b2: b0,
            del1: 0.0,
            del2: 0.0,
            k,
            samplerate: sr,
            cutoff_mod: None,
            q_mod: None,
        }
    }

    fn update_internals(&mut self, cutoff: f32, q: f32) {
        self.k = ((std::f32::consts::PI * cutoff) / self.samplerate).tanh();
        let k_pow_two = self.k.powf(2.0);
        self.a1 = (2.0 * q * (k_pow_two - 1.0)) / ((k_pow_two * q) + self.k + q);
        self.a2 = ((k_pow_two * q) - self.k + q) / ((k_pow_two * q) + self.k + q);
        self.b0 = q / ((k_pow_two * q) + self.k + q);
        self.b1 = -1.0 * ((2.0 * q) / ((k_pow_two * q) + self.k + q));
        self.b2 = self.b0;
    }
}

impl<const BUFSIZE: usize> MonoEffect<BUFSIZE> for BiquadHpf<BUFSIZE> {
    // some parameter limits might be nice ...
    fn set_parameter(&mut self, par: SynthParameterLabel, value: &SynthParameterValue) {
        match value {
            SynthParameterValue::ScalarF32(val) => {
                match par {
                    SynthParameterLabel::HighpassCutoffFrequency => self.cutoff = *val,
                    SynthParameterLabel::HighpassQFactor => self.q = *val,
                    _ => (),
                };

                self.update_internals(self.cutoff, self.q);
            }
            SynthParameterValue::Lfo(init, freq, eff_phase, amp, add, op) => {
                match par {
                    SynthParameterLabel::HighpassCutoffFrequency => {
                        self.cutoff = *init;
                        self.cutoff_mod = Some(Modulator::lfo(
                            *op,
                            *freq,
                            *eff_phase,
                            *amp,
                            *add,
                            true,
                            false,
                            self.samplerate,
                        ));
                    }
                    SynthParameterLabel::HighpassQFactor => {
                        self.q = *init;
                        self.q_mod = Some(Modulator::lfo(
                            *op,
                            *freq,
                            *eff_phase,
                            *amp,
                            *add,
                            true,
                            false,
                            self.samplerate,
                        ));
                    }
                    _ => (),
                };
                self.update_internals(self.cutoff, self.q);
            }
            SynthParameterValue::LFSaw(init, freq, amp, add, op) => {
                match par {
                    SynthParameterLabel::HighpassCutoffFrequency => {
                        self.cutoff = *init;
                        self.cutoff_mod = Some(Modulator::lfsaw(
                            *op,
                            *freq,
                            *amp,
                            *add,
                            true,
                            false,
                            self.samplerate,
                        ));
                    }
                    SynthParameterLabel::HighpassQFactor => {
                        self.q = *init;
                        self.q_mod = Some(Modulator::lfsaw(
                            *op,
                            *freq,
                            *amp,
                            *add,
                            true,
                            false,
                            self.samplerate,
                        ));
                    }
                    _ => (),
                };
                self.update_internals(self.cutoff, self.q);
            }
            SynthParameterValue::LFTri(init, freq, amp, add, op) => {
                match par {
                    SynthParameterLabel::HighpassCutoffFrequency => {
                        self.cutoff = *init;
                        self.cutoff_mod = Some(Modulator::lftri(
                            *op,
                            *freq,
                            *amp,
                            *add,
                            true,
                            false,
                            self.samplerate,
                        ));
                    }
                    SynthParameterLabel::HighpassQFactor => {
                        self.q = *init;
                        self.q_mod = Some(Modulator::lftri(
                            *op,
                            *freq,
                            *amp,
                            *add,
                            true,
                            false,
                            self.samplerate,
                        ));
                    }
                    _ => (),
                };
                self.update_internals(self.cutoff, self.q);
            }
            SynthParameterValue::LFSquare(init, freq, pw, amp, add, op) => {
                match par {
                    SynthParameterLabel::HighpassCutoffFrequency => {
                        self.cutoff = *init;
                        self.cutoff_mod = Some(Modulator::lfsquare(
                            *op,
                            *freq,
                            *pw,
                            *amp,
                            *add,
                            true,
                            false,
                            self.samplerate,
                        ));
                    }
                    SynthParameterLabel::HighpassQFactor => {
                        self.q = *init;
                        self.q_mod = Some(Modulator::lfsquare(
                            *op,
                            *freq,
                            *pw,
                            *amp,
                            *add,
                            true,
                            false,
                            self.samplerate,
                        ));
                    }
                    _ => (),
                };
                self.update_internals(self.cutoff, self.q);
            }
            _ => {}
        }
    }

    fn finish(&mut self) {} // this effect is stateless
    fn is_finished(&self) -> bool {
        false
    } // it's never finished ..

    // start sample isn't really needed either ...
    fn process_block(
        &mut self,
        block: [f32; BUFSIZE],
        start_sample: usize,
        in_buffers: &[Vec<f32>],
    ) -> [f32; BUFSIZE] {
        let mut out_buf: [f32; BUFSIZE] = [0.0; BUFSIZE];

        if self.cutoff_mod.is_some() || self.q_mod.is_some() {
            let cutoff_buf = if let Some(m) = self.cutoff_mod.as_mut() {
                m.process(self.cutoff, start_sample, in_buffers)
            } else {
                [self.cutoff; BUFSIZE]
            };

            let q_buf = if let Some(m) = self.q_mod.as_mut() {
                m.process(self.q, start_sample, in_buffers)
            } else {
                [self.q; BUFSIZE]
            };

            for i in start_sample..BUFSIZE {
                self.update_internals(cutoff_buf[i], q_buf[i]);

                let intermediate =
                    block[i] + ((-1.0 * self.a1) * self.del1) + ((-1.0 * self.a2) * self.del2);
                out_buf[i] =
                    (self.b0 * intermediate) + (self.b1 * self.del1) + (self.b2 * self.del2);
                self.del2 = self.del1;
                self.del1 = intermediate;
            }
        } else {
            for i in 0..BUFSIZE {
                let intermediate =
                    block[i] + ((-1.0 * self.a1) * self.del1) + ((-1.0 * self.a2) * self.del2);
                out_buf[i] =
                    (self.b0 * intermediate) + (self.b1 * self.del1) + (self.b2 * self.del2);
                self.del2 = self.del1;
                self.del1 = intermediate;
            }
        }

        out_buf
    }
}
