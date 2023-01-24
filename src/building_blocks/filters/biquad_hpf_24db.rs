use crate::building_blocks::{Modulator, MonoEffect, SynthParameterLabel, SynthParameterValue};

use crate::building_blocks::filters::sos::*;
use crate::building_blocks::filters::BiquadHpf12dB;

/**
 * Biquad HiPass Filter, 24dB/oct, cascaded second-order sections
 */
pub struct BiquadHpf24dB<const BUFSIZE: usize> {
    // user parameters
    cutoff: f32,
    q: f32,

    // internal parameters
    coefs: SOSCoefs,
    delay1: SOSDelay,
    delay2: SOSDelay,
    samplerate: f32,

    // modulator slots
    cutoff_mod: Option<Modulator<BUFSIZE>>,
    q_mod: Option<Modulator<BUFSIZE>>,
}

impl<const BUFSIZE: usize> BiquadHpf24dB<BUFSIZE> {
    pub fn new(freq: f32, q: f32, sr: f32) -> Self {
        let mut coefs: SOSCoefs = SOSCoefs::default();

        BiquadHpf12dB::<BUFSIZE>::generate_coefs(&mut coefs, freq, q, sr);

        BiquadHpf24dB {
            cutoff: freq,
            q,
            coefs,
            delay1: SOSDelay::default(),
            delay2: SOSDelay::default(),
            samplerate: sr,
            cutoff_mod: None,
            q_mod: None,
        }
    }
}

impl<const BUFSIZE: usize> MonoEffect<BUFSIZE> for BiquadHpf24dB<BUFSIZE> {
    fn set_modulator(
        &mut self,
        par: SynthParameterLabel,
        init: f32,
        modulator: Modulator<BUFSIZE>,
    ) {
        let mut update = false;
        match par {
            SynthParameterLabel::HighpassCutoffFrequency => {
                self.cutoff = init;
                self.cutoff_mod = Some(modulator);
                update = true;
            }
            SynthParameterLabel::HighpassQFactor => {
                self.q = init;
                self.q_mod = Some(modulator);
                update = true;
            }
            _ => {}
        }

        if update {
            BiquadHpf12dB::<BUFSIZE>::generate_coefs(
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
                SynthParameterLabel::HighpassCutoffFrequency => {
                    self.cutoff = *val;
                    update = true;
                }
                SynthParameterLabel::HighpassQFactor => {
                    self.q = *val;
                    update = true;
                }
                _ => (),
            };
            if update {
                BiquadHpf12dB::<BUFSIZE>::generate_coefs(
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
                BiquadHpf12dB::<BUFSIZE>::generate_coefs(
                    &mut self.coefs,
                    cutoff_buf[i],
                    q_buf[i],
                    self.samplerate,
                );
                out_buf[i] = process_sos_sample(
                    &self.coefs,
                    &mut self.delay2,
                    process_sos_sample(&self.coefs, &mut self.delay1, block[i]),
                );
            }

            out_buf
        } else {
            process_sos_block::<BUFSIZE>(
                &self.coefs,
                &mut self.delay2,
                process_sos_block::<BUFSIZE>(&self.coefs, &mut self.delay1, block),
            )
        }
    }
}
