use crate::building_blocks::{
    Modulator, MonoSource, SynthParameterLabel, SynthParameterValue, ValueOrModulator,
};

use std::f32::consts::PI;

/**
 * A quasi-bandlimited triangle oscillator using fm synthesis, following:
 *
 * Peter Schoffhauzer - Synthesis of Quasi-Bandliminted Analog Waveforms
 * Using Frequency Modulation
 *
 * http://scp.web.elte.hu/papers/synthesis1.pdf
 */
#[derive(Clone)]
pub struct FMTri<const BUFSIZE: usize> {
    // user parameters
    freq: f32,
    amp: f32,

    // internal parameters
    samplerate: f32,
    osc1: f32,    // current output sample
    osc2: f32,    // current output sample
    phase: f32,   // phase accumulator
    w: f32,       // normalized frequency
    scaling: f32, // scaling amount
    dc_comp: f32, // DC compensation
    norm: f32,    // normalization

    // pre-calculated filter constants
    a0: f32,
    a1: f32,
    del: f32, // filter delay

    // modulator slots
    freq_mod: Option<Modulator<BUFSIZE>>, // allows modulating frequency ..
    amp_mod: Option<Modulator<BUFSIZE>>,  // and level
}

impl<const BUFSIZE: usize> FMTri<BUFSIZE> {
    pub fn new(freq: f32, amp: f32, samplerate: f32) -> Self {
        let w: f32 = freq / samplerate;
        let n: f32 = 0.5 - w;
        FMTri {
            freq,
            amp,
            samplerate,
            osc1: 0.0,                     // current output sample
            osc2: 0.0,                     // current output sample
            phase: 0.0,                    // phase accumulator
            w,                             // normalized frequency
            scaling: 13.0 * n * n * n * n, // scaling amount
            dc_comp: 0.376 - w * 0.752,    // DC compensation
            norm: 1.0 - 2.0 * w,           // normalization
            // pre-calculated filter constants
            a0: 2.5,
            a1: -1.5,
            del: 0.0, // filter delay
            freq_mod: None,
            amp_mod: None,
        }
    }

    #[inline(always)]
    pub fn update_internals(&mut self, freq: f32) {
        self.w = freq / self.samplerate;
        let n: f32 = 0.5 - self.w;
        self.scaling = 13.0 * n * n * n * n;
        self.dc_comp = 0.376 - self.w * 0.752;
        self.norm = 1.0 - 2.0 * self.w;
    }
}

impl<const BUFSIZE: usize> MonoSource<BUFSIZE> for FMTri<BUFSIZE> {
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
            _ => {}
        }
    }

    // some parameter limits might be nice ...
    fn set_parameter(&mut self, par: SynthParameterLabel, value: &SynthParameterValue) {
        match par {
            SynthParameterLabel::PitchFrequency => {
                if let SynthParameterValue::ScalarF32(f) = value {
                    self.freq = *f;
                    self.update_internals(*f);
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

            for (i, current_sample) in out_buf
                .iter_mut()
                .enumerate()
                .take(BUFSIZE)
                .skip(start_sample)
            {
                self.update_internals(freq_buf[i]);

                // phase accum
                self.phase += self.w;
                self.phase -= self.phase.floor();
                let iphase = 2.0 * self.phase - 1.0;

                // next sample
                // next sample
                self.osc1 = (self.osc1 + (PI * (iphase + self.scaling * self.osc1)).sin()) * 0.5;
                let out = self.osc1 - self.osc2;
                *current_sample = (self.a0 * out - self.a1 * self.del + self.norm) * amp_buf[i];
                self.del = out;
            }
        } else {
            for current_sample in out_buf.iter_mut().take(BUFSIZE).skip(start_sample) {
                // phase accum
                self.phase += self.w;
                self.phase -= self.phase.floor();
                let iphase = 2.0 * self.phase - 1.0;

                // next sample
                self.osc1 = (self.osc1 + (PI * (iphase + self.scaling * self.osc1)).sin()) * 0.5;
                let mut out = self.a0 * self.osc1 - self.a1 * self.del;
                self.del = self.osc1;
                out += self.dc_comp * self.norm;
                out = (f32::min(out, -out) + 0.5) * 2.0;
                *current_sample = out * self.amp;
            }
        }

        out_buf
    }
}
