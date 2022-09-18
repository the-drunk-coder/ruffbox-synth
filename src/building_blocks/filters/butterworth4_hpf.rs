use crate::building_blocks::{
    Modulator, MonoEffect, SynthParameterLabel, SynthParameterValue, ValueOrModulator,
};

use crate::building_blocks::filters::sos::*;
use crate::building_blocks::filters::BiquadHpf12dB;

/**
 * Biquad LowPass Filter, 12dB/oct
 */
pub struct Butterworth4Hpf<const BUFSIZE: usize> {
    // user parameters
    cutoff: f32,

    // internal parameters
    coefs1: SOSCoefs,
    coefs2: SOSCoefs,
    delay1: SOSDelay,
    delay2: SOSDelay,
    samplerate: f32,

    // modulator slots
    cutoff_mod: Option<Modulator<BUFSIZE>>,
}

impl<const BUFSIZE: usize> Butterworth4Hpf<BUFSIZE> {
    pub fn new(freq: f32, sr: f32) -> Self {
        let mut coefs1: SOSCoefs = SOSCoefs::default();
        let mut coefs2: SOSCoefs = SOSCoefs::default();
        BiquadHpf12dB::<BUFSIZE>::generate_coefs(&mut coefs1, freq, 0.924, sr);
        BiquadHpf12dB::<BUFSIZE>::generate_coefs(&mut coefs2, freq, 0.383, sr);

        Butterworth4Hpf {
            cutoff: freq,
            coefs1,
            coefs2,
            delay1: SOSDelay::default(),
            delay2: SOSDelay::default(),
            samplerate: sr,
            cutoff_mod: None,
        }
    }
}

#[allow(clippy::single_match)]
impl<const BUFSIZE: usize> MonoEffect<BUFSIZE> for Butterworth4Hpf<BUFSIZE> {
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
            SynthParameterLabel::LowpassCutoffFrequency => {
                self.cutoff = init;
                self.cutoff_mod = Some(modulator);
            }
            _ => {}
        }
        BiquadHpf12dB::<BUFSIZE>::generate_coefs(
            &mut self.coefs1,
            self.cutoff,
            0.924,
            self.samplerate,
        );
        BiquadHpf12dB::<BUFSIZE>::generate_coefs(
            &mut self.coefs2,
            self.cutoff,
            0.383,
            self.samplerate,
        );
    }
    // some parameter limits might be nice ...
    fn set_parameter(&mut self, par: SynthParameterLabel, value: &SynthParameterValue) {
        if let SynthParameterValue::ScalarF32(val) = value {
            match par {
                SynthParameterLabel::LowpassCutoffFrequency => self.cutoff = *val,
                _ => (),
            };
            BiquadHpf12dB::<BUFSIZE>::generate_coefs(
                &mut self.coefs1,
                self.cutoff,
                0.924,
                self.samplerate,
            );
            BiquadHpf12dB::<BUFSIZE>::generate_coefs(
                &mut self.coefs2,
                self.cutoff,
                0.383,
                self.samplerate,
            );
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
        if self.cutoff_mod.is_some() {
            let mut out_buf: [f32; BUFSIZE] = [0.0; BUFSIZE];

            let cutoff_buf = if let Some(m) = self.cutoff_mod.as_mut() {
                m.process(self.cutoff, start_sample, in_buffers)
            } else {
                [self.cutoff; BUFSIZE]
            };

            for i in start_sample..BUFSIZE {
                BiquadHpf12dB::<BUFSIZE>::generate_coefs(
                    &mut self.coefs1,
                    cutoff_buf[i],
                    0.924,
                    self.samplerate,
                );
                BiquadHpf12dB::<BUFSIZE>::generate_coefs(
                    &mut self.coefs2,
                    cutoff_buf[i],
                    0.383,
                    self.samplerate,
                );
                out_buf[i] = process_sos_sample(
                    &self.coefs2,
                    &mut self.delay2,
                    process_sos_sample(&self.coefs1, &mut self.delay1, block[i]),
                );
            }

            out_buf
        } else {
            process_sos_block::<BUFSIZE>(
                &self.coefs2,
                &mut self.delay2,
                process_sos_block::<BUFSIZE>(&self.coefs1, &mut self.delay1, block),
            )
        }
    }
}
