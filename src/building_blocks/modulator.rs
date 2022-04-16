use crate::building_blocks::oscillators::*;
use crate::building_blocks::{MonoSource, SynthParameterLabel, ValOp};

/// modulate things ...
pub struct Modulator<const BUFSIZE: usize> {
    pub modulators: Vec<Modulator<BUFSIZE>>, // this will remain empty for now ...
    pub source: Box<dyn MonoSource<BUFSIZE> + Sync + Send>,
    pub param: SynthParameterLabel,
    pub op: ValOp,
    pub outlet_block: [f32; BUFSIZE],
    pub ext_override: bool,
}

impl<const BUFSIZE: usize> Modulator<BUFSIZE> {
    /// init lfo (sine) modulator
    pub fn lfo(
        param: SynthParameterLabel,
        op: ValOp,
        freq: f32,
        range: f32,
        sr: f32,
    ) -> Modulator<BUFSIZE> {
        Modulator {
            modulators: Vec::new(),
            source: Box::new(SineOsc::new(freq, range, sr)),
            param,
            op,
            outlet_block: [0.0; BUFSIZE],
	    ext_override: false,
        }
    }

    pub fn process(&mut self, start_sample: usize, in_buffers: &[Vec<f32>]) {
	if !self.ext_override {
	    self.outlet_block = self.source.get_next_block(start_sample, in_buffers);
	}
    }
}
