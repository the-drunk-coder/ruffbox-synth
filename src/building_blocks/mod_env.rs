use crate::building_blocks::{
    Modulator, MonoSource, SynthParameterLabel, SynthParameterValue, SynthState,
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

        for i in start_sample..BUFSIZE {
            out[i] = self.cur_lvl;

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

        for i in start_sample..BUFSIZE {
            out[i] = self.from + self.cur_lvl * self.mul;
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

    fn set_modulator(&mut self, _: SynthParameterLabel, _: f32, _: Modulator<BUFSIZE>) {}

    fn set_parameter(&mut self, _: SynthParameterLabel, _: &SynthParameterValue) {}

    fn get_next_block(&mut self, start_sample: usize, _: &[Vec<f32>]) -> [f32; BUFSIZE] {
        let mut out: [f32; BUFSIZE] = [0.0; BUFSIZE];

        for i in start_sample..BUFSIZE {
            out[i] = self.from + self.cur_lvl * self.mul;
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

pub enum SegmentType {
    Lin,
    Log,
    Exp,
}

pub struct SegmentInfo {
    pub from: f32,
    pub to: f32,
    pub time: f32,
    pub segment_type: SegmentType,
}

/**
 * Exponential Ramp
 */
#[derive(Clone)]
pub struct MultiPointEnvelope<const BUFSIZE: usize> {
    segments: Vec<Box<dyn MonoSource<BUFSIZE> + Sync + Send>>,
    segment_samples: Vec<usize>,
    segment_idx: usize,
    sample_count: usize,
    loop_env: bool,
    state: SynthState,
}

impl<const BUFSIZE: usize> MultiPointEnvelope<BUFSIZE> {
    pub fn new(segment_infos: Vec<SegmentInfo>, loop_env: bool, samplerate: f32) -> Self {
        let mut segments: Vec<Box<dyn MonoSource<BUFSIZE> + Sync + Send>> = Vec::new();
        let mut segment_samples = Vec::new();

        for info in segment_infos.iter() {
            segment_samples.push((info.time * samplerate).round() as usize);
            segments.push(match info.segment_type {
                SegmentType::Lin => {
                    Box::new(LinearRamp::new(info.from, info.to, info.time, samplerate))
                }
                SegmentType::Log => {
                    Box::new(LogRamp::new(info.from, info.to, info.time, samplerate))
                }
                SegmentType::Exp => {
                    Box::new(ExpRamp::new(info.from, info.to, info.time, samplerate))
                }
            });
        }

        MultiPointEnvelope {
            segments,
            segment_samples,
            segment_idx: 0,
            sample_count: 0,
            loop_env,
            state: SynthState::Fresh,
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

    fn finish(&mut self) {
        self.state = SynthState::Finished;
    }

    fn is_finished(&self) -> bool {
        false
    }

    fn set_modulator(&mut self, _: SynthParameterLabel, _: f32, _: Modulator<BUFSIZE>) {}

    fn set_parameter(&mut self, _: SynthParameterLabel, _: &SynthParameterValue) {}

    fn get_next_block(&mut self, start_sample: usize, bufs: &[Vec<f32>]) -> [f32; BUFSIZE] {
        // this should also avoid problems with "empty" multi-point envelopes ...
        if self.segment_idx >= self.segments.len() {
            if self.loop_env {
                self.reset();
            } else {
                if let Some(last_seg) = self.segments.last_mut() {
                    return last_seg.get_next_block(start_sample, bufs);
                } else {
                    return [0.0; BUFSIZE]; // last value ?
                }
            }
        }

        // what if there's 2 segments within one block ? --> IF we need to switch blocks, do this check !
        let samples_to_fill_total = BUFSIZE - start_sample;
        let mut samples_to_fill_rest = BUFSIZE - start_sample;
        let samples_left_in_segment = self.segment_samples[self.segment_idx] - self.sample_count;

        if samples_to_fill_total < samples_left_in_segment {
            self.sample_count += samples_to_fill_total;
            return self.segments[self.segment_idx].get_next_block(start_sample, bufs);
        } else {
            let mut out: [f32; BUFSIZE] = [0.0; BUFSIZE];

            let out_last = self.segments[self.segment_idx].get_next_block(start_sample, bufs);
            let mut left_from_current_segment =
                self.segment_samples[self.segment_idx] - self.sample_count;

            // handle leftover from current segment
            for i in start_sample..left_from_current_segment {
                out[i] = out_last[i]
            }

            samples_to_fill_rest -= left_from_current_segment;

            // we need some handling in case multiple segments fall into one block ...
            while samples_to_fill_rest > 0 {
                if let Some(next_segment) = self.segments.get_mut(self.segment_idx + 1) {
                    let next_segment_samples = self.segment_samples[self.segment_idx + 1];
                    if next_segment_samples >= samples_to_fill_rest {
                        let out_next = next_segment.get_next_block(left_from_current_segment, bufs);

                        for i in left_from_current_segment..samples_to_fill_total {
                            out[i] = out_next[i]
                        }

                        self.sample_count = samples_to_fill_rest - left_from_current_segment;
                        samples_to_fill_rest = 0;
                    } else {
                        let out_next = next_segment.get_next_block(left_from_current_segment, bufs);

                        for i in left_from_current_segment
                            ..left_from_current_segment + next_segment_samples
                        {
                            out[i] = out_next[i]
                        }

                        samples_to_fill_rest -= next_segment_samples;
                        left_from_current_segment += next_segment_samples;
                        self.sample_count = 0;
                    }
                } else {
                    self.sample_count = 0;
                    samples_to_fill_rest = 0;
                }

                self.segment_idx += 1;
            }

            return out;
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
            SegmentInfo {
                from: 0.0,
                to: 200.0,
                time: 0.003,
                segment_type: SegmentType::Lin,
            },
            SegmentInfo {
                from: 200.0,
                to: 100.0,
                time: 0.05,
                segment_type: SegmentType::Exp,
            },
            SegmentInfo {
                from: 100.0,
                to: 0.0,
                time: 0.03,
                segment_type: SegmentType::Log,
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
