use crate::building_blocks::{MonoEffect, SynthParameterLabel, SynthParameterValue};

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
        }
    }
}

impl<const BUFSIZE: usize> MonoEffect<BUFSIZE> for BiquadHpf<BUFSIZE> {
    // some parameter limits might be nice ...
    fn set_parameter(&mut self, par: SynthParameterLabel, value: &SynthParameterValue) {
        if let SynthParameterValue::ScalarF32(val) = value {
            match par {
                SynthParameterLabel::HighpassCutoffFrequency => self.cutoff = *val,
                SynthParameterLabel::HighpassQFactor => self.q = *val,
                _ => (),
            };

            // reset delay
            self.del1 = 0.0;
            self.del2 = 0.0;

            self.k = ((std::f32::consts::PI * self.cutoff) / self.samplerate).tanh();
            let k_pow_two = self.k.powf(2.0);
            self.a1 = (2.0 * self.q * (k_pow_two - 1.0)) / ((k_pow_two * self.q) + self.k + self.q);
            self.a2 =
                ((k_pow_two * self.q) - self.k + self.q) / ((k_pow_two * self.q) + self.k + self.q);
            self.b0 = self.q / ((k_pow_two * self.q) + self.k + self.q);
            self.b1 = -1.0 * ((2.0 * self.q) / ((k_pow_two * self.q) + self.k + self.q));
            self.b2 = self.b0;
        }
    }

    fn finish(&mut self) {} // this effect is stateless
    fn is_finished(&self) -> bool {
        false
    } // it's never finished ..

    // start sample isn't really needed either ...
    fn process_block(&mut self, block: [f32; BUFSIZE], _: usize) -> [f32; BUFSIZE] {
        let mut out_buf: [f32; BUFSIZE] = [0.0; BUFSIZE];

        for i in 0..BUFSIZE {
            let intermediate =
                block[i] + ((-1.0 * self.a1) * self.del1) + ((-1.0 * self.a2) * self.del2);
            out_buf[i] = (self.b0 * intermediate) + (self.b1 * self.del1) + (self.b2 * self.del2);
            self.del2 = self.del1;
            self.del1 = intermediate;
        }

        out_buf
    }
}
