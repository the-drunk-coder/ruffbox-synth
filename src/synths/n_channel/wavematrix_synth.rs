use crate::building_blocks::envelopes::*;
use crate::building_blocks::filters::*;
use crate::building_blocks::oscillators::wavematrix::Wavematrix;
use crate::building_blocks::routing::PanChan;
use crate::building_blocks::{
    Modulator, MonoEffect, MonoSource, Synth, SynthParameterLabel, SynthParameterValue,
    ValueOrModulator,
};

/// a simple wavetable synth with envelope etc.
pub struct WavematrixSynth<const BUFSIZE: usize, const NCHAN: usize> {
    wavematrix: Wavematrix<BUFSIZE>,
    envelope: LinearASREnvelope<BUFSIZE>,
    hpf: BiquadHpf12dB<BUFSIZE>,
    lpf: Lpf18<BUFSIZE>,
    balance: PanChan<BUFSIZE, NCHAN>,
    reverb: f32,
    delay: f32,
}

impl<const BUFSIZE: usize, const NCHAN: usize> WavematrixSynth<BUFSIZE, NCHAN> {
    pub fn new(sr: f32) -> WavematrixSynth<BUFSIZE, NCHAN> {
        WavematrixSynth {
            wavematrix: Wavematrix::new(sr),
            envelope: LinearASREnvelope::new(1.0, 0.0001, 0.1, 0.0001, sr),
            hpf: BiquadHpf12dB::new(20.0, 0.3, sr),
            lpf: Lpf18::new(19500.0, 0.01, 0.01, sr),
            balance: PanChan::new(),
            reverb: 0.0,
            delay: 0.0,
        }
    }
}

impl<const BUFSIZE: usize, const NCHAN: usize> Synth<BUFSIZE, NCHAN>
    for WavematrixSynth<BUFSIZE, NCHAN>
{
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
        self.wavematrix.set_modulator(par, init, modulator.clone());
        self.hpf.set_modulator(par, init, modulator.clone());
        self.lpf.set_modulator(par, init, modulator.clone());
        self.envelope.set_modulator(par, init, modulator.clone());
        self.balance.set_modulator(par, init, modulator);
    }

    fn set_parameter(&mut self, par: SynthParameterLabel, val: &SynthParameterValue) {
        self.wavematrix.set_parameter(par, val);
        self.hpf.set_parameter(par, val);
        self.lpf.set_parameter(par, val);
        self.envelope.set_parameter(par, val);
        self.balance.set_parameter(par, val);

        match par {
            SynthParameterLabel::ReverbMix => {
                if let SynthParameterValue::ScalarF32(r) = val {
                    self.reverb = *r
                }
            }
            SynthParameterLabel::DelayMix => {
                if let SynthParameterValue::ScalarF32(d) = val {
                    self.delay = *d
                }
            }
            _ => (),
        };
    }

    fn finish(&mut self) {
        self.envelope.finish();
    }

    fn is_finished(&self) -> bool {
        self.envelope.is_finished()
    }

    fn get_next_block(
        &mut self,
        start_sample: usize,
        sample_buffers: &[Vec<f32>],
    ) -> [[f32; BUFSIZE]; NCHAN] {
        let mut out: [f32; BUFSIZE] = self.wavematrix.get_next_block(start_sample, sample_buffers);
        out = self.hpf.process_block(out, start_sample, sample_buffers);
        out = self.lpf.process_block(out, start_sample, sample_buffers);
        out = self
            .envelope
            .process_block(out, start_sample, sample_buffers);
        self.balance
            .process_block(out, start_sample, sample_buffers) // needs the additional info for the modulators
    }

    fn reverb_level(&self) -> f32 {
        self.reverb
    }

    fn delay_level(&self) -> f32 {
        self.delay
    }
}
