use crate::building_blocks::oscillators::*;
use crate::building_blocks::{MonoSource, SynthParameterLabel};

/// what to do with a modulator ??
#[derive(Clone)]
pub enum ModulatorOperation {
    Replace,
    Add,
    Subtract,
    Multiply,
    Divide,
}

/// modulate things ...
pub struct Modulator<const BUFSIZE: usize> {
    pub modulators: Vec<Modulator<BUFSIZE>>, // this will remain empty for now ...
    pub source: Box<dyn MonoSource<BUFSIZE> + Sync + Send>,
    pub param: SynthParameterLabel,
    pub op: ModulatorOperation,
    pub outlet_block: [f32; BUFSIZE],
}

impl<const BUFSIZE: usize> Modulator<BUFSIZE> {
    /// init lfo (sine) modulator
    pub fn lfo(
        param: SynthParameterLabel,
        op: ModulatorOperation,
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
        }
    }

    pub fn process(&mut self, start_sample: usize, in_buffers: &[Vec<f32>]) {
        self.outlet_block = self.source.get_next_block(start_sample, in_buffers);
    }
}
