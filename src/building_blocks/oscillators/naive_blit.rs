use crate::building_blocks::{
    Modulator, MonoSource, SampleBuffer, SynthParameterLabel, SynthParameterValue,
};

use std::f32::consts::PI;

/**
 * A non-optimized BLIT implementation, following the stilson&smith paper,
 * some implementation details borrowed from the STK implementation.
 *
 * http://www.music.mcgill.ca/~gary/307/week5/node14.html
 * https://github.com/thestk/stk/blob/master/include/Blit.h
 * https://github.com/thestk/stk/blob/master/src/Blit.cpp
 */
#[derive(Clone)]
pub struct NaiveBlitOsc<const BUFSIZE: usize> {
    // user parameters
    freq: f32,
    amp: f32,
    num_harm: f32,

    // internal parameters
    m: f32,
    phase: f32,
    phase_inc: f32,
    p: f32,
    sr: f32,
}

impl<const BUFSIZE: usize> NaiveBlitOsc<BUFSIZE> {
    pub fn new(freq: f32, amp: f32, sr: f32) -> Self {
        let p = freq / sr;
        NaiveBlitOsc {
            freq,
            amp,
            m: 23.0,
            num_harm: 10.0,
            phase: 0.0,
            phase_inc: PI / p,
            p,
            sr,
        }
    }

    fn update_harmonics(&mut self) {
        let max_harmonics = (0.5 * self.p).floor();
        // if set to zero, use maximum available harmonics,
        // otherwise, use provided number
        if self.num_harm == 0.0 {
            self.m = 2.0 * max_harmonics + 1.0; // number of harmonics is always odd
        } else {
            self.m = 2.0 * self.num_harm.floor() + 1.0; // number of harmonics is always odd
        }
    }
}

#[inline(always)]
// phase already includes the pi multiplication
// this isn't quite the sinc_m function from the paper,
// but this one works and the other one doesn't??
fn sinc_ish_m(phase: f32, m: f32) -> f32 {
    let den_f = phase.sin();
    if den_f < f32::EPSILON {
        1.0 // avoid division by zero, value approaches 1 in this case
    } else {
        (phase * m).sin() / (m * den_f)
    }
}

impl<const BUFSIZE: usize> MonoSource<BUFSIZE> for NaiveBlitOsc<BUFSIZE> {
    fn reset(&mut self) {}

    fn set_modulator(
        &mut self,
        _par: SynthParameterLabel,
        _init: f32,
        _modulator: Modulator<BUFSIZE>,
    ) {
    }
    // some parameter limits might be nice ...
    fn set_parameter(&mut self, par: SynthParameterLabel, value: &SynthParameterValue) {
        match par {
            SynthParameterLabel::PitchFrequency => {
                if let SynthParameterValue::ScalarF32(f) = value {
                    self.freq = *f;
                    self.p = self.sr / *f;
                    self.phase_inc = PI / self.p;
                    self.update_harmonics();
                }
            }
            SynthParameterLabel::NumHarmonics => {
                if let SynthParameterValue::ScalarF32(f) = value {
                    // not clamping harmonics, so aliasing might still
                    // occur if not chosen correctly ...
                    self.num_harm = *f;
                    self.update_harmonics();
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

    fn get_next_block(
        &mut self,
        start_sample: usize,
        _in_buffers: &[SampleBuffer],
    ) -> [f32; BUFSIZE] {
        let mut out_buf: [f32; BUFSIZE] = [0.0; BUFSIZE];

        // the closed-from BLIT formula:
        // BLIT(x) = (M/P) * sin(M * pi * x) / M sin(pi * x);
        // the phase arguments to the sin functions are replaced
        // M = 2 * numHarmonics + 1 (maxHarmonics = floor( 0.5 * p))
        // p = sr / freq
        // with the phase, which is incremented by PI/p
        // the (M/P) factor is reversed to produce a normalized signal

        for current_sample in out_buf.iter_mut().take(BUFSIZE).skip(start_sample) {
            *current_sample = sinc_ish_m(self.phase, self.m) * self.amp;

            // keep phase in [-PI;PI]
            self.phase += self.phase_inc;
            if self.phase >= PI {
                self.phase -= PI;
            }
        }

        out_buf
    }
}
