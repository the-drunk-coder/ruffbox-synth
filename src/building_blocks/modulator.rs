use crate::building_blocks::oscillators::*;
use crate::building_blocks::{MonoSource, ValOp};

/// modulate things ...
pub struct Modulator<const BUFSIZE: usize> {
    pub source: Box<dyn MonoSource<BUFSIZE> + Sync + Send>,
    pub op: ValOp,
}

impl<const BUFSIZE: usize> Modulator<BUFSIZE> {
    /// init lfo (sine) modulator
    pub fn lfo(op: ValOp, freq: f32, range: f32, sr: f32) -> Modulator<BUFSIZE> {
        Modulator {
            source: Box::new(SineOsc::new(freq, range, sr)),
            op,
        }
    }

    pub fn process(
        &mut self,
        original_value: f32,
        start_sample: usize,
        in_buffers: &[Vec<f32>],
    ) -> [f32; BUFSIZE] {
        match self.op {
            ValOp::Add => self
                .source
                .get_next_block(start_sample, in_buffers)
                .map(|x| original_value + x),
            ValOp::Subtract => self
                .source
                .get_next_block(start_sample, in_buffers)
                .map(|x| original_value - x),
            ValOp::Multiply => self
                .source
                .get_next_block(start_sample, in_buffers)
                .map(|x| original_value * x),
            ValOp::Divide => self
                .source
                .get_next_block(start_sample, in_buffers)
                .map(|x| original_value / x),
            _ => self.source.get_next_block(start_sample, in_buffers),
        }
    }
}
