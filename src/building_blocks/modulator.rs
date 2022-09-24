use crate::building_blocks::mod_env::*;
use crate::building_blocks::oscillators::*;
use crate::building_blocks::{
    EnvelopeSegmentInfo, MonoSource, SynthParameterLabel, SynthParameterValue, ValOp,
    ValueOrModulator,
};

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
        freq: ValueOrModulator<BUFSIZE>,
        eff_phase: f32,
        amp: ValueOrModulator<BUFSIZE>,
        add: f32,
        positive: bool,
        rectify: bool,
        sr: f32,
    ) -> Modulator<BUFSIZE> {
        let mut src_osc = SineOsc::new(5.0, 1.0, sr);

        src_osc.set_param_or_modulator(SynthParameterLabel::PitchFrequency, freq);
        src_osc.set_param_or_modulator(SynthParameterLabel::OscillatorAmplitude, amp);
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
    #[allow(clippy::too_many_arguments)]
    pub fn lfsaw(
        op: ValOp,
        freq: ValueOrModulator<BUFSIZE>,
        eff_phase: f32,
        amp: ValueOrModulator<BUFSIZE>,
        add: f32,
        positive: bool,
        rectify: bool,
        sr: f32,
    ) -> Modulator<BUFSIZE> {
        let mut src_osc = LFSaw::new(5.0, 1.0, sr);
        src_osc.set_param_or_modulator(SynthParameterLabel::PitchFrequency, freq);
        src_osc.set_param_or_modulator(SynthParameterLabel::OscillatorAmplitude, amp);
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
    #[allow(clippy::too_many_arguments)]
    pub fn lfrsaw(
        op: ValOp,
        freq: ValueOrModulator<BUFSIZE>,
        eff_phase: f32,
        amp: ValueOrModulator<BUFSIZE>,
        add: f32,
        positive: bool,
        rectify: bool,
        sr: f32,
    ) -> Modulator<BUFSIZE> {
        let mut src_osc = LFRSaw::new(5.0, 1.0, sr);
        src_osc.set_param_or_modulator(SynthParameterLabel::PitchFrequency, freq);
        src_osc.set_param_or_modulator(SynthParameterLabel::OscillatorAmplitude, amp);
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
    #[allow(clippy::too_many_arguments)]
    pub fn lftri(
        op: ValOp,
        freq: ValueOrModulator<BUFSIZE>,
        eff_phase: f32,
        amp: ValueOrModulator<BUFSIZE>,
        add: f32,
        positive: bool,
        rectify: bool,
        sr: f32,
    ) -> Modulator<BUFSIZE> {
        let mut src_osc = LFTri::new(5.0, 1.0, sr);
        src_osc.set_param_or_modulator(SynthParameterLabel::PitchFrequency, freq);
        src_osc.set_param_or_modulator(SynthParameterLabel::OscillatorAmplitude, amp);
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
        freq: ValueOrModulator<BUFSIZE>,
        pw: f32,
        amp: ValueOrModulator<BUFSIZE>,
        add: f32,
        positive: bool,
        rectify: bool,
        sr: f32,
    ) -> Modulator<BUFSIZE> {
        let mut src_osc = LFSquare::new(5.0, pw, 1.0, sr);
        src_osc.set_param_or_modulator(SynthParameterLabel::PitchFrequency, freq);
        src_osc.set_param_or_modulator(SynthParameterLabel::OscillatorAmplitude, amp);
        Modulator {
            source: Box::new(src_osc),
            op,
            add,
            positive,
            rectify,
        }
    }

    /// init linear ramp modulator
    pub fn lin_ramp(op: ValOp, from: f32, to: f32, time: f32, sr: f32) -> Modulator<BUFSIZE> {
        Modulator {
            source: Box::new(LinearRamp::new(from, to, time, sr)),
            op,
            add: 0.0,
            positive: false,
            rectify: false,
        }
    }

    /// init logarithmic ramp modulator
    pub fn log_ramp(op: ValOp, from: f32, to: f32, time: f32, sr: f32) -> Modulator<BUFSIZE> {
        Modulator {
            source: Box::new(LogRamp::new(from, to, time, sr)),
            op,
            add: 0.0,
            positive: false,
            rectify: false,
        }
    }

    /// init exponential ramp modulator
    pub fn exp_ramp(op: ValOp, from: f32, to: f32, time: f32, sr: f32) -> Modulator<BUFSIZE> {
        Modulator {
            source: Box::new(ExpRamp::new(from, to, time, sr)),
            op,
            add: 0.0,
            positive: false,
            rectify: false,
        }
    }

    /// init multi-point envelope modulator
    pub fn multi_point_envelope(
        op: ValOp,
        segments: Vec<EnvelopeSegmentInfo>,
        loop_env: bool,
        sr: f32,
    ) -> Modulator<BUFSIZE> {
        Modulator {
            source: Box::new(MultiPointEnvelope::new(segments, loop_env, sr)),
            op,
            add: 0.0,
            positive: false,
            rectify: false,
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
    fn test_modulator_range_square() {
        let mut modulator = Modulator::<512>::lfsquare(
            ValOp::Replace,
            ValueOrModulator::Val(SynthParameterValue::ScalarF32(1.0)),
            0.1,
            ValueOrModulator::Val(SynthParameterValue::ScalarF32(1.0)),
            0.0,
            false,
            false,
            44100.0,
        );

        for _ in 0..200 {
            let block = modulator.process(1.0, 0, &Vec::new());
            for i in 0..512 {
                let a = block[i];
                debug_plotter::plot!(a where caption = "SQRPlot");
            }
        }
    }
}
