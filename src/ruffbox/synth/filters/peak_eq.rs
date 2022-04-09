use crate::ruffbox::synth::MonoEffect;
use crate::ruffbox::synth::{SynthParameterLabel, SynthParameterValue};

/**
 * Peak/Eq Filter
 */
pub struct PeakEq<const BUFSIZE: usize> {
    // user parameters
    center: f32,
    bw: f32,
    gain: f32,

    // internal parameters
    h_zero: f32,
    v_zero: f32,
    d: f32,
    del1: f32,
    del2: f32,
    c: f32,
    samplerate: f32,
}

impl<const BUFSIZE: usize> PeakEq<BUFSIZE> {
    pub fn new(center_freq: f32, bw: f32, gain: f32, sr: f32) -> Self {
        let w_c = (2.0 * center_freq) / sr;
        let w_b = (2.0 * bw) / sr;
        let d = -((std::f32::consts::PI * w_c).cos());
        let v_zero = (gain / 20.0).powf(10.0);
        let h_zero = v_zero - 1.0;
        let cf_tan = (std::f32::consts::PI * w_b / 2.0).tan();

        let c = if gain >= 0.0 {
            (cf_tan - 1.0) / (cf_tan + 1.0)
        } else {
            (cf_tan - v_zero) / (cf_tan + v_zero)
        };

        PeakEq {
            center: center_freq,
            bw,
            gain,
            h_zero,
            v_zero,
            d,
            del1: 0.0,
            del2: 0.0,
            c,
            samplerate: sr,
        }
    }
}

impl<const BUFSIZE: usize> MonoEffect<BUFSIZE> for PeakEq<BUFSIZE> {
    // some parameter limits might be nice ...
    fn set_parameter(&mut self, par: SynthParameterLabel, value: SynthParameterValue) {
        if let SynthParameterValue::ScalarF32(val) = value {
            match par {
                SynthParameterLabel::PeakFrequency => self.center = val,
                SynthParameterLabel::PeakGain => self.gain = val,
                SynthParameterLabel::PeakQFactor => self.bw = val,
                _ => (),
            };

            // reset delay
            self.del1 = 0.0;
            self.del2 = 0.0;

            let w_c = (2.0 * self.center) / self.samplerate;
            let w_b = (2.0 * self.bw) / self.samplerate;
            self.d = -((std::f32::consts::PI * w_c).cos());
            self.v_zero = (10.0_f32).powf(self.gain / 20.0);
            self.h_zero = self.v_zero - 1.0;
            let cf_tan = (std::f32::consts::PI * w_b / 2.0).tan();

            self.c = if self.gain >= 0.0 {
                (cf_tan - 1.0) / (cf_tan + 1.0)
            } else {
                (cf_tan - self.v_zero) / (cf_tan + self.v_zero)
            };
        }
    }

    fn finish(&mut self) {} // this effect is stateless
    fn is_finished(&self) -> bool {
        false
    } // it's never finished ..

    // start sample isn't really needed either ...
    fn process_block(&mut self, block: [f32; BUFSIZE], _start_sample: usize) -> [f32; BUFSIZE] {
        let mut out_buf: [f32; BUFSIZE] = [0.0; BUFSIZE];

        for i in 0..BUFSIZE {
            let x_h = block[i] - self.d * (1.0 - self.c) * self.del1 + (self.c * self.del2);
            let y_one = (-1.0 * self.c * x_h) + (self.d * (1.0 - self.c) * self.del1) + self.del2;
            out_buf[i] = 0.5 * self.h_zero * (block[i] - y_one) + block[i];
            self.del2 = self.del1;
            self.del1 = x_h;
        }

        out_buf
    }
}
