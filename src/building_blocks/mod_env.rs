use crate::building_blocks::{
    EnvelopeSegmentInfo, EnvelopeSegmentType, Modulator, MonoSource, SynthParameterLabel,
    SynthParameterValue, SynthState, ValueOrModulator,
};

/**
 * Linear Ramp
 */
#[derive(Clone, Copy)]
pub struct LinearRamp<const BUFSIZE: usize> {
    ramp_samples: usize,
    sample_count: usize,
    cur_lvl: f32,
    to: f32,
    from: f32,
    inc_dec: f32,
    state: SynthState,
}

impl<const BUFSIZE: usize> LinearRamp<BUFSIZE> {
    pub fn new(from: f32, to: f32, ramp_time: f32, samplerate: f32) -> Self {
        let ramp_samples = (samplerate * ramp_time).round();

        LinearRamp {
            ramp_samples: ramp_samples as usize,
            sample_count: 0,
            cur_lvl: from,
            to,
            from,
            inc_dec: (to - from) / ramp_samples,
            state: SynthState::Fresh,
        }
    }
}

impl<const BUFSIZE: usize> MonoSource<BUFSIZE> for LinearRamp<BUFSIZE> {
    fn reset(&mut self) {
        self.sample_count = 0;
        self.cur_lvl = self.from;
    }

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

    fn finish(&mut self) {
        self.state = SynthState::Finished;
    }

    fn is_finished(&self) -> bool {
        false
    }

    fn set_modulator(&mut self, _: SynthParameterLabel, _: f32, _: Modulator<BUFSIZE>) {}

    fn set_parameter(&mut self, _: SynthParameterLabel, _: &SynthParameterValue) {}

    fn get_next_block(&mut self, start_sample: usize, _: &[Vec<f32>]) -> [f32; BUFSIZE] {
        let mut out: [f32; BUFSIZE] = [0.0; BUFSIZE];

        for current_sample in out.iter_mut().take(BUFSIZE).skip(start_sample) {
            *current_sample = self.cur_lvl;

            if self.sample_count < self.ramp_samples {
                self.cur_lvl += self.inc_dec;
            } else {
                self.cur_lvl = self.to;
            }

            self.sample_count += 1;
        }
        out
    }
}

/**
 * Logarithmic Ramp
 */
#[derive(Clone, Copy)]
pub struct LogRamp<const BUFSIZE: usize> {
    ramp_samples: usize,
    sample_count: usize,
    time_inc: f32,
    time_count: f32,
    curve: f32,
    cur_lvl: f32,
    mul: f32,
    from: f32,
    state: SynthState,
}

impl<const BUFSIZE: usize> LogRamp<BUFSIZE> {
    pub fn new(from: f32, to: f32, ramp_time: f32, samplerate: f32) -> Self {
        let ramp_samples = (samplerate * ramp_time).round();
        let time_inc = 1.0 / ramp_samples;
        let mul = to - from;

        LogRamp {
            ramp_samples: ramp_samples as usize,
            sample_count: 0,
            cur_lvl: 0.0,
            time_inc,
            curve: -4.5 * mul.signum(),
            time_count: 0.0,
            mul,
            from,
            state: SynthState::Fresh,
        }
    }
}

impl<const BUFSIZE: usize> MonoSource<BUFSIZE> for LogRamp<BUFSIZE> {
    fn reset(&mut self) {
        self.sample_count = 0;
        self.cur_lvl = self.from;
    }

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

    fn finish(&mut self) {
        self.state = SynthState::Finished;
    }

    fn is_finished(&self) -> bool {
        false
    }

    fn set_modulator(&mut self, _: SynthParameterLabel, _: f32, _: Modulator<BUFSIZE>) {}

    fn set_parameter(&mut self, _: SynthParameterLabel, _: &SynthParameterValue) {}

    fn get_next_block(&mut self, start_sample: usize, _: &[Vec<f32>]) -> [f32; BUFSIZE] {
        let mut out: [f32; BUFSIZE] = [0.0; BUFSIZE];

        for current_sample in out.iter_mut().take(BUFSIZE).skip(start_sample) {
            *current_sample = self.from + self.cur_lvl * self.mul;
            self.cur_lvl = if self.sample_count < self.ramp_samples {
                ((self.curve * self.time_count).exp() - 1.0) / (self.curve.exp() - 1.0)
            } else {
                1.0
            };

            self.time_count += self.time_inc;

            self.sample_count += 1;
        }
        out
    }
}

/**
 * Exponential Ramp
 */
#[derive(Clone, Copy)]
pub struct ExpRamp<const BUFSIZE: usize> {
    ramp_samples: usize,
    sample_count: usize,
    time_inc: f32,
    time_count: f32,
    curve: f32,
    cur_lvl: f32,
    mul: f32,
    from: f32,
    state: SynthState,
}

impl<const BUFSIZE: usize> ExpRamp<BUFSIZE> {
    pub fn new(from: f32, to: f32, ramp_time: f32, samplerate: f32) -> Self {
        let ramp_samples = (samplerate * ramp_time).round();
        let time_inc = 1.0 / ramp_samples;
        let mul = to - from;

        ExpRamp {
            ramp_samples: ramp_samples as usize,
            sample_count: 0,
            cur_lvl: 0.0,
            time_inc,
            curve: 4.5 * mul.signum(),
            time_count: 0.0,
            mul,
            from,
            state: SynthState::Fresh,
        }
    }
}

impl<const BUFSIZE: usize> MonoSource<BUFSIZE> for ExpRamp<BUFSIZE> {
    fn reset(&mut self) {
        self.sample_count = 0;
        self.cur_lvl = self.from;
    }

    fn finish(&mut self) {
        self.state = SynthState::Finished;
    }

    fn is_finished(&self) -> bool {
        false
    }

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

    fn set_modulator(&mut self, _: SynthParameterLabel, _: f32, _: Modulator<BUFSIZE>) {}

    fn set_parameter(&mut self, _: SynthParameterLabel, _: &SynthParameterValue) {}

    fn get_next_block(&mut self, start_sample: usize, _: &[Vec<f32>]) -> [f32; BUFSIZE] {
        let mut out: [f32; BUFSIZE] = [0.0; BUFSIZE];

        for current_sample in out.iter_mut().take(BUFSIZE).skip(start_sample) {
            *current_sample = self.from + self.cur_lvl * self.mul;
            self.cur_lvl = if self.sample_count < self.ramp_samples {
                ((self.curve * self.time_count).exp() - 1.0) / (self.curve.exp() - 1.0)
            } else {
                1.0
            };

            self.time_count += self.time_inc;

            self.sample_count += 1;
        }
        out
    }
}

/**
 * Constant (needed for envelope)
 */
#[derive(Clone, Copy)]
pub struct ConstantMod<const BUFSIZE: usize> {
    value: f32,
    state: SynthState,
}

impl<const BUFSIZE: usize> ConstantMod<BUFSIZE> {
    pub fn new(value: f32) -> Self {
        ConstantMod {
            value,
            state: SynthState::Fresh,
        }
    }
}

impl<const BUFSIZE: usize> MonoSource<BUFSIZE> for ConstantMod<BUFSIZE> {
    fn reset(&mut self) {}

    fn finish(&mut self) {
        self.state = SynthState::Finished;
    }

    fn is_finished(&self) -> bool {
        false
    }

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

    fn set_modulator(&mut self, _: SynthParameterLabel, _: f32, _: Modulator<BUFSIZE>) {}

    fn set_parameter(&mut self, _: SynthParameterLabel, _: &SynthParameterValue) {}

    fn get_next_block(&mut self, _: usize, _: &[Vec<f32>]) -> [f32; BUFSIZE] {
        [self.value; BUFSIZE]
    }
}

/**
 * Multi-Point Modulator Envelope
 */
#[derive(Clone)]
pub struct MultiPointEnvelope<const BUFSIZE: usize> {
    segments: Vec<Box<dyn MonoSource<BUFSIZE> + Sync + Send>>,
    segment_samples: Vec<usize>,
    segment_idx: usize,
    sample_count: usize, // re-set on every segment switch
    loop_env: bool,
    state: SynthState,
    samplerate: f32,
}

impl<const BUFSIZE: usize> MultiPointEnvelope<BUFSIZE> {
    pub fn new(segment_infos: Vec<EnvelopeSegmentInfo>, loop_env: bool, samplerate: f32) -> Self {
        let mut segments: Vec<Box<dyn MonoSource<BUFSIZE> + Sync + Send>> = Vec::new();
        let mut segment_samples = Vec::new();

        for info in segment_infos.iter() {
            segment_samples.push((info.time * samplerate).round() as usize);
            segments.push(match info.segment_type {
                EnvelopeSegmentType::Lin => {
                    Box::new(LinearRamp::new(info.from, info.to, info.time, samplerate))
                }
                EnvelopeSegmentType::Log => {
                    Box::new(LogRamp::new(info.from, info.to, info.time, samplerate))
                }
                EnvelopeSegmentType::Exp => {
                    Box::new(ExpRamp::new(info.from, info.to, info.time, samplerate))
                }
                EnvelopeSegmentType::Constant => Box::new(ConstantMod::new(info.to)),
            });
        }

        MultiPointEnvelope {
            segments,
            segment_samples,
            segment_idx: 0,
            sample_count: 0,
            loop_env,
            state: SynthState::Fresh,
            samplerate,
        }
    }

    pub fn empty(samplerate: f32) -> Self {
        MultiPointEnvelope {
            segments: Vec::new(),
            segment_samples: Vec::new(),
            segment_idx: 0,
            sample_count: 0,
            loop_env: false,
            state: SynthState::Fresh,
            samplerate,
        }
    }
}

impl<const BUFSIZE: usize> MonoSource<BUFSIZE> for MultiPointEnvelope<BUFSIZE> {
    fn reset(&mut self) {
        self.sample_count = 0;
        self.segment_idx = 0;
        for s in self.segments.iter_mut() {
            s.reset();
        }
    }

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

    fn finish(&mut self) {
        self.state = SynthState::Finished;
    }

    fn is_finished(&self) -> bool {
        if let Some(last) = self.segments.last() {
            // check if last element has finished or whether this is a looping envelope
            !self.loop_env && last.is_finished()
        } else {
            true // an empty envelope doesn't do anything and is always finished
        }
    }

    fn set_modulator(&mut self, _: SynthParameterLabel, _: f32, _: Modulator<BUFSIZE>) {}

    fn set_parameter(&mut self, par: SynthParameterLabel, val: &SynthParameterValue) {
        // TODO: recalc envelope segments from attack, decay, sustain, release etc ...
        if let SynthParameterLabel::Envelope = par {
            if let SynthParameterValue::MultiPointEnvelope(segment_infos, loop_env, _) = val {
                let mut segments: Vec<Box<dyn MonoSource<BUFSIZE> + Sync + Send>> = Vec::new();
                let mut segment_samples = Vec::new();

                for info in segment_infos.iter() {
                    segment_samples.push((info.time * self.samplerate).round() as usize);
                    segments.push(match info.segment_type {
                        EnvelopeSegmentType::Lin => Box::new(LinearRamp::new(
                            info.from,
                            info.to,
                            info.time,
                            self.samplerate,
                        )),
                        EnvelopeSegmentType::Log => {
                            Box::new(LogRamp::new(info.from, info.to, info.time, self.samplerate))
                        }
                        EnvelopeSegmentType::Exp => {
                            Box::new(ExpRamp::new(info.from, info.to, info.time, self.samplerate))
                        }
                        EnvelopeSegmentType::Constant => Box::new(ConstantMod::new(info.to)),
                    });
                }

                self.segments = segments;
                self.segment_samples = segment_samples;
                self.loop_env = *loop_env;
            }
        }
    }

    fn get_next_block(&mut self, start_sample: usize, bufs: &[Vec<f32>]) -> [f32; BUFSIZE] {
        // this should also avoid problems with "empty" multi-point envelopes ...
        if self.segment_idx >= self.segments.len() {
            if let Some(last_seg) = self.segments.last_mut() {
                // only loop a
                if self.loop_env && last_seg.is_finished() {
                    self.reset();
                } else {
                    return last_seg.get_next_block(start_sample, bufs);
                }
            } else {
                // this means this in an empty envelope ...
                return [0.0; BUFSIZE]; // last value ?
            }
        }

        // first, let's see how many samples we have to fill
        let block_samples_to_fill_total = BUFSIZE - start_sample;

        // now, let's see how many samples we have left in the current segment ...
        let samples_left_in_segment = self.segment_samples[self.segment_idx] - self.sample_count;

        // if we have more samples to fill in current segment than we need for current
        // block, great, that's the easiest case ...
        if block_samples_to_fill_total < samples_left_in_segment {
            self.sample_count += block_samples_to_fill_total;
            // ... because we just need to return whatever the current segment gives us ...
            self.segments[self.segment_idx].get_next_block(start_sample, bufs)
        } else {
            // otherwise, we need to check how many samples we need to fill
            // and assemble the out buffer piece by piece
            let mut out: [f32; BUFSIZE] = [0.0; BUFSIZE];

            // countdown ...
            let mut block_samples_to_fill_rest = block_samples_to_fill_total;

            // leftovers from the segment we were currently handling ...
            let mut out_current =
                self.segments[self.segment_idx].get_next_block(start_sample, bufs);
            let left_from_current_segment =
                self.segment_samples[self.segment_idx] - self.sample_count;

            // the first piece of the block ...
            out[start_sample..start_sample + left_from_current_segment].copy_from_slice(
                &out_current[start_sample..start_sample + left_from_current_segment],
            );

            // re-set sample count as the current segment
            // has been handled completely ...
            self.sample_count = 0;

            // is there anything more to fill ??
            block_samples_to_fill_rest -= left_from_current_segment;

            // how far are we advanced in the current block ?
            let mut start_index = start_sample + left_from_current_segment;

            // we need some handling in case multiple segments fall into one block,
            // so we count down on the samples that are left to fill ...
            while block_samples_to_fill_rest > 0 {
                // if there is a next segment ...
                if let Some(next_segment) = self.segments.get_mut(self.segment_idx + 1) {
                    // .. let's see how long it is ...
                    let next_segment_samples = self.segment_samples[self.segment_idx + 1];

                    out_current = next_segment.get_next_block(start_index, bufs);

                    // again, more than we need ?
                    if next_segment_samples >= block_samples_to_fill_rest {
                        // copy samples
                        out[start_index..block_samples_to_fill_total].copy_from_slice(
                            &out_current[start_index..block_samples_to_fill_total],
                        );

                        self.sample_count = block_samples_to_fill_total - start_index;
                        self.segment_idx += 1; // we're in the next segment now ...
                        break; // jump out
                    } else {
                        // copy samples
                        out[start_index..(start_index + next_segment_samples)].copy_from_slice(
                            &out_current[start_index..(start_index + next_segment_samples)],
                        );

                        block_samples_to_fill_rest -= next_segment_samples;
                        start_index += next_segment_samples;
                        self.segment_idx += 1; // we're in the next segment now ...
                        self.sample_count = 0; // re-set sample count as we finished a segment ...
                    }
                } else if self.loop_env {
                    // continue filling the block after resetting the segment counter
                    // and the individual segments
                    self.reset();
                    continue;
                } else {
                    // there is no next segment, but still something to fill ...
                    // copy samples from whatever was left over from last block ...
                    out[start_index..block_samples_to_fill_total]
                        .copy_from_slice(&out_current[start_index..block_samples_to_fill_total]);
                    self.sample_count = 0;
                    self.segment_idx += 1; // we're in the next segment now ...
                    break; // jump out
                }
            }
            // return the piecewise-assembled out block ...
            out
        }
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use crate::building_blocks::MonoSource;

    #[test]
    fn test_lin_ramp() {
        let mut linramp = LinearRamp::<512>::new(200.0, 10.0, 2.0, 44100.0);

        let num_blocks = (88200.0 / 512.0) as usize + 30;

        for _ in 0..num_blocks {
            let block = linramp.get_next_block(0, &Vec::new());
            for i in 0..512 {
                let a = block[i];
                debug_plotter::plot!(a  where caption = "LinRampRevTest");
            }
        }
    }

    #[test]
    fn test_log_ramp() {
        let mut logramp = LogRamp::<512>::new(-20.0, -200.0, 2.0, 44100.0);

        let num_blocks = (88200.0 / 512.0) as usize + 30;

        let block = logramp.get_next_block(256, &Vec::new());
        for i in 0..512 {
            let a = block[i];
            debug_plotter::plot!(a where caption = "ExpRampTest");
        }

        for _ in 1..num_blocks {
            let block = logramp.get_next_block(0, &Vec::new());
            for i in 0..512 {
                let a = block[i];
                debug_plotter::plot!(a  where caption = "ExpRampTest");
            }
        }
    }

    #[test]
    fn test_exp_ramp() {
        let mut expramp = ExpRamp::<512>::new(200.0, 20.0, 2.0, 44100.0);

        let num_blocks = (88200.0 / 512.0) as usize + 30;

        for _ in 0..num_blocks {
            let block = expramp.get_next_block(0, &Vec::new());
            for i in 0..512 {
                let a = block[i];
                debug_plotter::plot!(a  where caption = "ExpRampTest");
            }
        }
    }

    #[test]
    fn test_multi_point() {
        let segments = vec![
            EnvelopeSegmentInfo {
                from: 0.0,
                to: 200.0,
                time: 0.003,
                segment_type: EnvelopeSegmentType::Lin,
            },
            EnvelopeSegmentInfo {
                from: 200.0,
                to: 100.0,
                time: 0.05,
                segment_type: EnvelopeSegmentType::Exp,
            },
            EnvelopeSegmentInfo {
                from: 100.0,
                to: 0.0,
                time: 0.03,
                segment_type: EnvelopeSegmentType::Log,
            },
        ];

        let mut mpenv = MultiPointEnvelope::<2048>::new(segments, false, 44100.0);
        let num_blocks = (0.2 * 44100.0 / 2048.0) as usize;

        for _ in 0..num_blocks {
            let block = mpenv.get_next_block(0, &Vec::new());
            for i in 0..2048 {
                let a = block[i];
                debug_plotter::plot!(a where caption = "MultiPointTest");
            }
        }
    }
}
