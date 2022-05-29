use crate::building_blocks::oscillators::*;
use crate::building_blocks::{MonoSource, ValOp};

/// modulate things ...
pub struct Modulator<const BUFSIZE: usize> {
    pub source: Box<dyn MonoSource<BUFSIZE> + Sync + Send>,
    pub op: ValOp,
    pub add: f32,
}

impl<const BUFSIZE: usize> Modulator<BUFSIZE> {
    /// init lfo (sine) modulator
    pub fn lfo(op: ValOp, freq: f32, amp: f32, add: f32, sr: f32) -> Modulator<BUFSIZE> {
        Modulator {
            source: Box::new(SineOsc::new(freq, amp, sr)),
            op,
            add,
        }
    }

    /// init lfo (sine) modulator
    pub fn lfsaw(op: ValOp, freq: f32, amp: f32, add: f32, sr: f32) -> Modulator<BUFSIZE> {
        Modulator {
            source: Box::new(LFSaw::new(freq, amp, sr)),
            op,
            add,
        }
    }

    /// init lfo (sine) modulator
    pub fn lftri(op: ValOp, freq: f32, amp: f32, add: f32, sr: f32) -> Modulator<BUFSIZE> {
        Modulator {
            source: Box::new(LFTri::new(freq, amp, sr)),
            op,
            add,
        }
    }

    /// init lfo (sine) modulator
    pub fn lfsquare(
        op: ValOp,
        freq: f32,
        pw: f32,
        amp: f32,
        add: f32,
        sr: f32,
    ) -> Modulator<BUFSIZE> {
        Modulator {
            source: Box::new(LFSquare::new(freq, pw, amp, sr)),
            op,
            add,
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
