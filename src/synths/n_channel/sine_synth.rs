use crate::building_blocks::envelopes::*;
use crate::building_blocks::oscillators::*;
use crate::building_blocks::routing::PanChan;
use crate::building_blocks::{
    MonoEffect, MonoSource, Synth, SynthParameterLabel, SynthParameterValue,
};

/// a sinusoidal synth with envelope etc.
pub struct SineSynth<const BUFSIZE: usize, const NCHAN: usize> {
    oscillator: SineOsc<BUFSIZE>,
    envelope: LinearASREnvelope<BUFSIZE>,
    balance: PanChan<BUFSIZE, NCHAN>,
    reverb: f32,
    delay: f32,
}

impl<const BUFSIZE: usize, const NCHAN: usize> SineSynth<BUFSIZE, NCHAN> {
    pub fn new(sr: f32) -> Self {
        SineSynth {
            oscillator: SineOsc::new(440.0, 0.5, sr),
            envelope: LinearASREnvelope::new(0.3, 0.05, 0.1, 0.05, sr),
            balance: PanChan::new(sr),
            reverb: 0.0,
            delay: 0.0,
        }
    }
}

impl<const BUFSIZE: usize, const NCHAN: usize> Synth<BUFSIZE, NCHAN> for SineSynth<BUFSIZE, NCHAN> {
    fn set_parameter(&mut self, par: SynthParameterLabel, val: &SynthParameterValue) {
        self.oscillator.set_parameter(par, val);
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
        let mut out: [f32; BUFSIZE] = self.oscillator.get_next_block(start_sample, sample_buffers);
        out = self.envelope.process_block(out, start_sample);
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
