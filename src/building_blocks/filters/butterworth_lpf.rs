use crate::building_blocks::{
    Modulator, MonoEffect, SynthParameterLabel, SynthParameterValue,
};

use crate::building_blocks::filters::sos::*;
use crate::building_blocks::filters::BiquadLpf12dB;

/**
 * Second-Order-Section based Butterworth filter with variable order ranging from 2 to 10,
 * modeled with the values by dafx ...
 */
pub struct ButterworthLpf<const BUFSIZE: usize> {
    // user parameters
    cutoff: f32,

    // internal parameters
    order: usize,
    coefs: Vec<SOSCoefs>,
    delays: Vec<SOSDelay>,

    samplerate: f32,

    // modulator slots
    cutoff_mod: Option<Modulator<BUFSIZE>>,
}

impl<const BUFSIZE: usize> ButterworthLpf<BUFSIZE> {
    pub fn new(freq: f32, order: usize, sr: f32) -> Self {
        let mut coefs = Vec::new();
        let mut delays = Vec::new();

        match order {
            2 => {
                let mut coefs1: SOSCoefs = SOSCoefs::default();
                BiquadLpf12dB::<BUFSIZE>::generate_coefs(&mut coefs1, freq, 0.707, sr);
                coefs.push(coefs1);
                delays.push(SOSDelay::default());
            }
            4 => {
                let mut coefs1: SOSCoefs = SOSCoefs::default();
                let mut coefs2: SOSCoefs = SOSCoefs::default();
                BiquadLpf12dB::<BUFSIZE>::generate_coefs(&mut coefs1, freq, 0.924, sr);
                BiquadLpf12dB::<BUFSIZE>::generate_coefs(&mut coefs2, freq, 0.383, sr);
                coefs.push(coefs1);
                coefs.push(coefs2);
                delays.push(SOSDelay::default());
                delays.push(SOSDelay::default());
            }
            6 => {
                let mut coefs1: SOSCoefs = SOSCoefs::default();
                let mut coefs2: SOSCoefs = SOSCoefs::default();
                let mut coefs3: SOSCoefs = SOSCoefs::default();
                BiquadLpf12dB::<BUFSIZE>::generate_coefs(&mut coefs1, freq, 0.966, sr);
                BiquadLpf12dB::<BUFSIZE>::generate_coefs(&mut coefs2, freq, 0.707, sr);
                BiquadLpf12dB::<BUFSIZE>::generate_coefs(&mut coefs3, freq, 0.259, sr);
                coefs.push(coefs1);
                coefs.push(coefs2);
                coefs.push(coefs3);
                delays.push(SOSDelay::default());
                delays.push(SOSDelay::default());
                delays.push(SOSDelay::default());
            }
            8 => {
                let mut coefs1: SOSCoefs = SOSCoefs::default();
                let mut coefs2: SOSCoefs = SOSCoefs::default();
                let mut coefs3: SOSCoefs = SOSCoefs::default();
                let mut coefs4: SOSCoefs = SOSCoefs::default();
                BiquadLpf12dB::<BUFSIZE>::generate_coefs(&mut coefs1, freq, 0.981, sr);
                BiquadLpf12dB::<BUFSIZE>::generate_coefs(&mut coefs2, freq, 0.831, sr);
                BiquadLpf12dB::<BUFSIZE>::generate_coefs(&mut coefs3, freq, 0.556, sr);
                BiquadLpf12dB::<BUFSIZE>::generate_coefs(&mut coefs4, freq, 0.195, sr);
                coefs.push(coefs1);
                coefs.push(coefs2);
                coefs.push(coefs3);
                coefs.push(coefs4);
                delays.push(SOSDelay::default());
                delays.push(SOSDelay::default());
                delays.push(SOSDelay::default());
                delays.push(SOSDelay::default());
            }
            10 => {
                let mut coefs1: SOSCoefs = SOSCoefs::default();
                let mut coefs2: SOSCoefs = SOSCoefs::default();
                let mut coefs3: SOSCoefs = SOSCoefs::default();
                let mut coefs4: SOSCoefs = SOSCoefs::default();
                let mut coefs5: SOSCoefs = SOSCoefs::default();
                BiquadLpf12dB::<BUFSIZE>::generate_coefs(&mut coefs1, freq, 0.988, sr);
                BiquadLpf12dB::<BUFSIZE>::generate_coefs(&mut coefs2, freq, 0.891, sr);
                BiquadLpf12dB::<BUFSIZE>::generate_coefs(&mut coefs3, freq, 0.707, sr);
                BiquadLpf12dB::<BUFSIZE>::generate_coefs(&mut coefs4, freq, 0.454, sr);
                BiquadLpf12dB::<BUFSIZE>::generate_coefs(&mut coefs5, freq, 0.156, sr);
                coefs.push(coefs1);
                coefs.push(coefs2);
                coefs.push(coefs3);
                coefs.push(coefs4);
                coefs.push(coefs5);
                delays.push(SOSDelay::default());
                delays.push(SOSDelay::default());
                delays.push(SOSDelay::default());
                delays.push(SOSDelay::default());
                delays.push(SOSDelay::default());
            }
            _ => {
                let mut coefs1: SOSCoefs = SOSCoefs::default();
                BiquadLpf12dB::<BUFSIZE>::generate_coefs(&mut coefs1, freq, 0.707, sr);
                coefs.push(coefs1);
                delays.push(SOSDelay::default());
            }
        }

        ButterworthLpf {
            cutoff: freq,
            coefs,
            delays,
            order,
            samplerate: sr,
            cutoff_mod: None,
        }
    }

    fn regenerate_coefs(&mut self, cutoff: f32) {
        match self.order {
            2 => {
                BiquadLpf12dB::<BUFSIZE>::generate_coefs(
                    &mut self.coefs[0],
                    cutoff,
                    0.707,
                    self.samplerate,
                );
            }
            4 => {
                BiquadLpf12dB::<BUFSIZE>::generate_coefs(
                    &mut self.coefs[0],
                    cutoff,
                    0.924,
                    self.samplerate,
                );
                BiquadLpf12dB::<BUFSIZE>::generate_coefs(
                    &mut self.coefs[1],
                    cutoff,
                    0.383,
                    self.samplerate,
                );
            }
            6 => {
                BiquadLpf12dB::<BUFSIZE>::generate_coefs(
                    &mut self.coefs[0],
                    cutoff,
                    0.966,
                    self.samplerate,
                );
                BiquadLpf12dB::<BUFSIZE>::generate_coefs(
                    &mut self.coefs[1],
                    cutoff,
                    0.707,
                    self.samplerate,
                );
                BiquadLpf12dB::<BUFSIZE>::generate_coefs(
                    &mut self.coefs[2],
                    cutoff,
                    0.259,
                    self.samplerate,
                );
            }
            8 => {
                BiquadLpf12dB::<BUFSIZE>::generate_coefs(
                    &mut self.coefs[0],
                    cutoff,
                    0.981,
                    self.samplerate,
                );
                BiquadLpf12dB::<BUFSIZE>::generate_coefs(
                    &mut self.coefs[1],
                    cutoff,
                    0.831,
                    self.samplerate,
                );
                BiquadLpf12dB::<BUFSIZE>::generate_coefs(
                    &mut self.coefs[2],
                    cutoff,
                    0.556,
                    self.samplerate,
                );
                BiquadLpf12dB::<BUFSIZE>::generate_coefs(
                    &mut self.coefs[3],
                    cutoff,
                    0.195,
                    self.samplerate,
                );
            }
            10 => {
                BiquadLpf12dB::<BUFSIZE>::generate_coefs(
                    &mut self.coefs[0],
                    cutoff,
                    0.988,
                    self.samplerate,
                );
                BiquadLpf12dB::<BUFSIZE>::generate_coefs(
                    &mut self.coefs[1],
                    cutoff,
                    0.891,
                    self.samplerate,
                );
                BiquadLpf12dB::<BUFSIZE>::generate_coefs(
                    &mut self.coefs[2],
                    cutoff,
                    0.707,
                    self.samplerate,
                );
                BiquadLpf12dB::<BUFSIZE>::generate_coefs(
                    &mut self.coefs[3],
                    cutoff,
                    0.454,
                    self.samplerate,
                );
                BiquadLpf12dB::<BUFSIZE>::generate_coefs(
                    &mut self.coefs[4],
                    cutoff,
                    0.156,
                    self.samplerate,
                );
            }
            _ => {
                BiquadLpf12dB::<BUFSIZE>::generate_coefs(
                    &mut self.coefs[0],
                    cutoff,
                    0.707,
                    self.samplerate,
                );
            }
        }
    }
}

#[allow(clippy::single_match)]
impl<const BUFSIZE: usize> MonoEffect<BUFSIZE> for ButterworthLpf<BUFSIZE> {
    
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
                self.regenerate_coefs(self.cutoff);
            }
            _ => {}
        }
        self.regenerate_coefs(self.cutoff);
    }
    // some parameter limits might be nice ...
    fn set_parameter(&mut self, par: SynthParameterLabel, value: &SynthParameterValue) {
        if let SynthParameterValue::ScalarF32(val) = value {
            match par {
                SynthParameterLabel::LowpassCutoffFrequency => {
                    self.cutoff = *val;
                    self.regenerate_coefs(self.cutoff);
                }
                _ => (),
            };
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
                self.regenerate_coefs(cutoff_buf[i]);

                out_buf[i] = process_sos_sample(&self.coefs[0], &mut self.delays[0], block[i]);
                for (j, coefs) in self.coefs.iter().enumerate().skip(1) {
                    out_buf[i] = process_sos_sample(coefs, &mut self.delays[j], out_buf[i])
                }
            }

            out_buf
        } else {
            let mut out_block =
                process_sos_block::<BUFSIZE>(&self.coefs[0], &mut self.delays[0], block);
            for (i, coefs) in self.coefs.iter().enumerate().skip(1) {
                out_block = process_sos_block::<BUFSIZE>(coefs, &mut self.delays[i], block)
            }
            out_block
        }
    }
}
