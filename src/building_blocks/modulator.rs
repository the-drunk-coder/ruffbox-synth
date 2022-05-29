use crate::building_blocks::oscillators::*;
use crate::building_blocks::{MonoSource, ValOp};

/// modulate things ...
pub struct Modulator<const BUFSIZE: usize> {
    pub source: Box<dyn MonoSource<BUFSIZE> + Sync + Send>,
    pub op: ValOp,
    pub add: f32,
    pub positive: bool,
    pub rectify: bool,
}

impl<const BUFSIZE: usize> Modulator<BUFSIZE> {
    /// init lfo (sine) modulator
    pub fn lfo(
        op: ValOp,
        freq: f32,
        amp: f32,
        add: f32,
        positive: bool,
        rectify: bool,
        sr: f32,
    ) -> Modulator<BUFSIZE> {
        Modulator {
            source: Box::new(SineOsc::new(freq, amp, sr)),
            op,
            add,
            positive,
            rectify,
        }
    }

    /// init saw modulator
    pub fn lfsaw(
        op: ValOp,
        freq: f32,
        amp: f32,
        add: f32,
        positive: bool,
        rectify: bool,
        sr: f32,
    ) -> Modulator<BUFSIZE> {
        Modulator {
            source: Box::new(LFSaw::new(freq, amp, sr)),
            op,
            add,
            positive,
            rectify,
        }
    }

    /// init tri modulator
    pub fn lftri(
        op: ValOp,
        freq: f32,
        amp: f32,
        add: f32,
        positive: bool,
        rectify: bool,
        sr: f32,
    ) -> Modulator<BUFSIZE> {
        Modulator {
            source: Box::new(LFTri::new(freq, amp, sr)),
            op,
            add,
            positive,
            rectify,
        }
    }

    /// init square modulator
    #[allow(clippy::too_many_arguments)]
    pub fn lfsquare(
        op: ValOp,
        freq: f32,
        pw: f32,
        amp: f32,
        add: f32,
        positive: bool,
        rectify: bool,
        sr: f32,
    ) -> Modulator<BUFSIZE> {
        Modulator {
            source: Box::new(LFSquare::new(freq, pw, amp, sr)),
            op,
            add,
            positive,
            rectify,
        }
    }

    pub fn process(
        &mut self,
        original_value: f32,
        start_sample: usize,
        in_buffers: &[Vec<f32>],
    ) -> [f32; BUFSIZE] {
        if self.positive {
            match self.op {
                ValOp::Add => self
                    .source
                    .get_next_block(start_sample, in_buffers)
                    .map(|x| f32::max(original_value + x + self.add, 0.0001)),
                ValOp::Subtract => self
                    .source
                    .get_next_block(start_sample, in_buffers)
                    .map(|x| f32::max(original_value - x + self.add, 0.0001)),
                ValOp::Multiply => self
                    .source
                    .get_next_block(start_sample, in_buffers)
                    .map(|x| f32::max(original_value * x + self.add, 0.0001)),
                ValOp::Divide => self
                    .source
                    .get_next_block(start_sample, in_buffers)
                    .map(|x| f32::max(original_value / x + self.add, 0.0001)),
                _ => self.source.get_next_block(start_sample, in_buffers),
            }
        } else if self.rectify {
            match self.op {
                ValOp::Add => self
                    .source
                    .get_next_block(start_sample, in_buffers)
                    .map(|x| (original_value + x + self.add).abs()),
                ValOp::Subtract => self
                    .source
                    .get_next_block(start_sample, in_buffers)
                    .map(|x| (original_value - x + self.add).abs()),
                ValOp::Multiply => self
                    .source
                    .get_next_block(start_sample, in_buffers)
                    .map(|x| (original_value * x + self.add).abs()),
                ValOp::Divide => self
                    .source
                    .get_next_block(start_sample, in_buffers)
                    .map(|x| (original_value / x + self.add).abs()),
                _ => self.source.get_next_block(start_sample, in_buffers),
            }
        } else {
            match self.op {
                ValOp::Add => self
                    .source
                    .get_next_block(start_sample, in_buffers)
                    .map(|x| original_value + x + self.add),
                ValOp::Subtract => self
                    .source
                    .get_next_block(start_sample, in_buffers)
                    .map(|x| original_value - x + self.add),
                ValOp::Multiply => self
                    .source
                    .get_next_block(start_sample, in_buffers)
                    .map(|x| original_value * x + self.add),
                ValOp::Divide => self
                    .source
                    .get_next_block(start_sample, in_buffers)
                    .map(|x| original_value / x + self.add),
                _ => self.source.get_next_block(start_sample, in_buffers),
            }
        }
    }
}
