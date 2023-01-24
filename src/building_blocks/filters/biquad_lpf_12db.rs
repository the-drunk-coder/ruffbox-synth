use crate::building_blocks::{Modulator, MonoEffect, SynthParameterLabel, SynthParameterValue};

use crate::building_blocks::filters::sos::*;

/**
 * Biquad LowPass Filter, 12dB/oct
 */
pub struct BiquadLpf12dB<const BUFSIZE: usize> {
    // user parameters
    cutoff: f32,
    q: f32,

    // internal parameters
    coefs: SOSCoefs,
    delay: SOSDelay,
    samplerate: f32,

    // modulator slots
    cutoff_mod: Option<Modulator<BUFSIZE>>,
    q_mod: Option<Modulator<BUFSIZE>>,
}

impl<const BUFSIZE: usize> BiquadLpf12dB<BUFSIZE> {
    pub fn new(freq: f32, q: f32, sr: f32) -> Self {
        let mut coefs: SOSCoefs = SOSCoefs::default();
        BiquadLpf12dB::<BUFSIZE>::generate_coefs(&mut coefs, freq, q, sr);

        BiquadLpf12dB {
            cutoff: freq,
            q,
            coefs,
            delay: SOSDelay::default(),
            samplerate: sr,
            cutoff_mod: None,
            q_mod: None,
        }
    }

    #[inline(always)]
    pub fn generate_coefs(coefs: &mut SOSCoefs, cutoff: f32, q: f32, sr: f32) {
        let k = ((std::f32::consts::PI * cutoff) / sr).tanh();
        let k_pow_two = k.powf(2.0);
        coefs.a1 = (2.0 * q * (k_pow_two - 1.0)) / ((k_pow_two * q) + k + q);
        coefs.a2 = ((k_pow_two * q) - k + q) / ((k_pow_two * q) + k + q);
        coefs.b0 = (k_pow_two * q) / ((k_pow_two * q) + k + q);
        coefs.b1 = (2.0 * k_pow_two * q) / ((k_pow_two * q) + k + q);
        coefs.b2 = coefs.b0;
    }
}

impl<const BUFSIZE: usize> MonoEffect<BUFSIZE> for BiquadLpf12dB<BUFSIZE> {
    fn set_modulator(
        &mut self,
        par: SynthParameterLabel,
        init: f32,
        modulator: Modulator<BUFSIZE>,
    ) {
        let mut update = false;
        match par {
            SynthParameterLabel::LowpassCutoffFrequency => {
                self.cutoff = init;
                self.cutoff_mod = Some(modulator);
                update = true;
            }
            SynthParameterLabel::LowpassQFactor => {
                self.q = init;
                self.q_mod = Some(modulator);
                update = true;
            }
            _ => {}
        }

        if update {
            BiquadLpf12dB::<BUFSIZE>::generate_coefs(
                &mut self.coefs,
                self.cutoff,
                self.q,
                self.samplerate,
            );
        }
    }
    // some parameter limits might be nice ...
    fn set_parameter(&mut self, par: SynthParameterLabel, value: &SynthParameterValue) {
        let mut update = false;
        if let SynthParameterValue::ScalarF32(val) = value {
            match par {
                SynthParameterLabel::LowpassCutoffFrequency => {
                    self.cutoff = *val;
                    update = true
                }
                SynthParameterLabel::LowpassQFactor => {
                    self.q = *val;
                    update = true;
                }
                _ => (),
            };

            if update {
                BiquadLpf12dB::<BUFSIZE>::generate_coefs(
                    &mut self.coefs,
                    self.cutoff,
                    self.q,
                    self.samplerate,
                );
            }
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
        in_buffers: &[SampleBuffer],
    ) -> [f32; BUFSIZE] {
        if self.cutoff_mod.is_some() || self.q_mod.is_some() {
            let mut out_buf: [f32; BUFSIZE] = [0.0; BUFSIZE];

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
                BiquadLpf12dB::<BUFSIZE>::generate_coefs(
                    &mut self.coefs,
                    cutoff_buf[i],
                    q_buf[i],
                    self.samplerate,
                );
                out_buf[i] = process_sos_sample(&self.coefs, &mut self.delay, block[i]);
            }

            out_buf
        } else {
            process_sos_block::<BUFSIZE>(&self.coefs, &mut self.delay, block)
        }
    }
}
