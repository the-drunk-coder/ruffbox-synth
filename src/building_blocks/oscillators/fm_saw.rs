use crate::building_blocks::{Modulator, MonoSource, SynthParameterLabel, SynthParameterValue};

use std::f32::consts::PI;

/**
 * A quasi-bandlimited sawtooth oscillator using fm synthesis, following:
 *
 * Peter Schoffhauzer - Synthesis of Quasi-Bandliminted Analog Waveforms
 * Using Frequency Modulation
 *
 * http://scp.web.elte.hu/papers/synthesis1.pdf
 */
#[derive(Clone)]
pub struct FMSaw<const BUFSIZE: usize> {
    // user parameters
    freq: f32,
    amp: f32,

    // internal parameters
    samplerate: f32,
    osc: f32,     // current output sample
    phase: f32,   // phase accumulator
    w: f32,       // normalized frequency
    scaling: f32, // scaling amount

    dc_comp: f32,
    norm: f32,
    del: f32, // one-pole filter delay

    // modulator slots
    freq_mod: Option<Modulator<BUFSIZE>>, // allows modulating frequency ..
    amp_mod: Option<Modulator<BUFSIZE>>,  // and level
}

impl<const BUFSIZE: usize> FMSaw<BUFSIZE> {
    pub fn new(freq: f32, amp: f32, samplerate: f32) -> Self {
        let w: f32 = freq / samplerate;
        let n: f32 = 0.5 - w;
        FMSaw {
            freq,
            amp,
            samplerate,
            osc: 0.0,                      // current output sample
            phase: 0.0,                    // phase accumulator
            w,                             // normalized frequency
            scaling: 13.0 * n * n * n * n, // scaling amount
            dc_comp: 0.1 + w * 0.2,
            norm: 1.0 - 2.0 * w,
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
        self.dc_comp = 0.1 + self.w * 0.2;
        self.norm = 1.0 - 2.0 * self.w;
    }
}

impl<const BUFSIZE: usize> MonoSource<BUFSIZE> for FMSaw<BUFSIZE> {
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

            for (i, current_sample) in out_buf
                .iter_mut()
                .enumerate()
                .take(BUFSIZE)
                .skip(start_sample)
            {
                self.update_internals(freq_buf[i]);

                // phase accum
                self.phase += 2.0 * self.w;
                if self.phase >= 1.0 {
                    self.phase -= 2.0;
                }

                // next sample
                // the paper says 2pi, but that doesn't make sense when you look at the plots ...
                self.osc = (self.osc + (PI * (self.phase + self.scaling * self.osc)).sin()) * 0.5;
                let o = 2.5 * self.osc - 1.5 * self.del;
                self.del = self.osc;
                // the normalization is different than in the paper, but it seems more symmetric
                // to me this way ..
                *current_sample = (o - self.dc_comp) * self.norm * amp_buf[i];
            }
        } else {
            for current_sample in out_buf.iter_mut().take(BUFSIZE).skip(start_sample) {
                // phase accum
                self.phase += 2.0 * self.w;
                if self.phase >= 1.0 {
                    self.phase -= 2.0;
                }

                // next sample
                // the paper says 2pi, but that doesn't make sense when you look at the plots ...
                self.osc = (self.osc + (PI * (self.phase + self.scaling * self.osc)).sin()) * 0.5;
                // one-pole lowpass filter
                let o = 2.5 * self.osc - 1.5 * self.del;
                self.del = self.osc;
                // the normalization is different than in the paper, but it seems more symmetric
                // to me this way ..
                *current_sample = (o - self.dc_comp) * self.norm * self.amp;
            }
        }

        out_buf
    }
}
