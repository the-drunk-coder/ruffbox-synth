use crate::building_blocks::oscillators::*;
use crate::building_blocks::{MonoSource, SynthParameterLabel, SynthParameterValue, ValOp};

/// modulate things ...
#[derive(Clone)]
pub struct Modulator<const BUFSIZE: usize> {
    pub source: Box<dyn MonoSource<BUFSIZE> + Sync + Send>,
    pub op: ValOp,
    pub add: f32,
    pub positive: bool,
    pub rectify: bool,
}

impl<const BUFSIZE: usize> Modulator<BUFSIZE> {
    /// init lfo (sine) modulator
    #[allow(clippy::too_many_arguments)]
    pub fn lfo(
        op: ValOp,
        freq: f32,
        eff_phase: f32,
        amp: f32,
        add: f32,
        positive: bool,
        rectify: bool,
        sr: f32,
    ) -> Modulator<BUFSIZE> {
        let mut src_osc = SineOsc::new(freq, amp, sr);
        src_osc.set_parameter(
            SynthParameterLabel::OscillatorPhaseEffective,
            &SynthParameterValue::ScalarF32(eff_phase - add),
        );
        Modulator {
            source: Box::new(src_osc),
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
        eff_phase: f32,
        amp: f32,
        add: f32,
        positive: bool,
        rectify: bool,
        sr: f32,
    ) -> Modulator<BUFSIZE> {
        let mut src_osc = LFSaw::new(freq, amp, sr);
        src_osc.set_parameter(
            SynthParameterLabel::OscillatorPhaseEffective,
            &SynthParameterValue::ScalarF32(eff_phase - add),
        );
        Modulator {
            source: Box::new(src_osc),
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
        eff_phase: f32,
        amp: f32,
        add: f32,
        positive: bool,
        rectify: bool,
        sr: f32,
    ) -> Modulator<BUFSIZE> {
        let mut src_osc = LFTri::new(freq, amp, sr);
        src_osc.set_parameter(
            SynthParameterLabel::OscillatorPhaseEffective,
            &SynthParameterValue::ScalarF32(eff_phase - add),
        );
        Modulator {
            source: Box::new(src_osc),
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
                ValOp::Replace => self
                    .source
                    .get_next_block(start_sample, in_buffers)
                    .map(|x| f32::max(x + self.add, 0.0001)),
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
                ValOp::Replace => self
                    .source
                    .get_next_block(start_sample, in_buffers)
                    .map(|x| (x + self.add).abs()),
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
                ValOp::Replace => self
                    .source
                    .get_next_block(start_sample, in_buffers)
                    .map(|x| x + self.add),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_modulator_range() {
        let mut modulator =
            Modulator::<512>::lftri(ValOp::Replace, 1.0, 3.0, 1.0, 3.0, false, false, 44100.0);

        for _ in 0..100 {
            let block = modulator.process(1.0, 0, &Vec::new());
            for i in 0..512 {
                let a = block[i];
                debug_plotter::plot!(a where caption = "LFOPlot");
            }
        }
    }
}
