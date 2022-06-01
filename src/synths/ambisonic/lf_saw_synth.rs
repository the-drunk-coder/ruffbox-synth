use crate::building_blocks::ambisonics::encoder_o1::EncoderO1;
use crate::building_blocks::envelopes::*;
use crate::building_blocks::filters::*;
use crate::building_blocks::oscillators::*;
use crate::building_blocks::{
    Modulator, MonoEffect, MonoSource, Synth, SynthParameterLabel, SynthParameterValue,
};

/// a low-frequency sawtooth synth with envelope and lpf18 filter
pub struct LFSawSynth<const BUFSIZE: usize> {
    oscillator: LFSaw<BUFSIZE>,
    filter: Lpf18<BUFSIZE>,
    envelope: LinearASREnvelope<BUFSIZE>,
    encoder: EncoderO1<BUFSIZE>,
    reverb: f32,
    delay: f32,
}

impl<const BUFSIZE: usize> LFSawSynth<BUFSIZE> {
    #[allow(dead_code)]
    pub fn new(sr: f32) -> Self {
        LFSawSynth {
            oscillator: LFSaw::new(100.0, 0.8, sr),
            filter: Lpf18::new(1500.0, 0.5, 0.1, sr),
            envelope: LinearASREnvelope::new(1.0, 0.002, 0.02, 0.08, sr),
            encoder: EncoderO1::new(),
            reverb: 0.0,
            delay: 0.0,
        }
    }
}

impl<const BUFSIZE: usize> Synth<BUFSIZE, 4> for LFSawSynth<BUFSIZE> {
    fn set_modulator(
        &mut self,
        par: SynthParameterLabel,
        init: f32,
        modulator: Modulator<BUFSIZE>,
    ) {
        self.oscillator.set_modulator(par, init, modulator.clone());
        self.filter.set_modulator(par, init, modulator.clone());
        self.envelope.set_modulator(par, init, modulator);
    }

    fn set_parameter(&mut self, par: SynthParameterLabel, val: &SynthParameterValue) {
        self.oscillator.set_parameter(par, val);
        self.filter.set_parameter(par, val);
        self.envelope.set_parameter(par, val);
        self.encoder.set_parameter(par, val);

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
    ) -> [[f32; BUFSIZE]; 4] {
        let mut out: [f32; BUFSIZE] = self.oscillator.get_next_block(start_sample, sample_buffers);
        out = self.filter.process_block(out, start_sample, sample_buffers);
        out = self
            .envelope
            .process_block(out, start_sample, sample_buffers);
        self.encoder.process_block(out)
    }

    fn reverb_level(&self) -> f32 {
        self.reverb
    }

    fn delay_level(&self) -> f32 {
        self.delay
    }
}
