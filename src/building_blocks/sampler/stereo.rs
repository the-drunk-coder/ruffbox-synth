// parent imports
use crate::building_blocks::{
    interpolation::*, Modulator, SampleBuffer, StereoSource, SynthParameterLabel,
    SynthParameterValue, SynthState,
};

/**
 * a very simple sample player ...
 */
#[derive(Clone)]
pub struct StereoSampler<const BUFSIZE: usize> {
    // user parameters
    playback_rate: f32,
    amp: f32,

    // internal parameters
    phase: usize,
    frac_phase: f32,
    bufnum: usize,
    buflen: usize,
    // pre-calc some often-used values
    buflen_plus_one: usize,
    buflen_plus_one_f32: f32,
    frac_phase_increment: f32,
    state: SynthState,
    repeat: bool,

    // modulator slots
    rate_mod: Option<Modulator<BUFSIZE>>,
    amp_mod: Option<Modulator<BUFSIZE>>,
}

impl<const BUFSIZE: usize> StereoSampler<BUFSIZE> {
    pub fn with_bufnum_len(bufnum: usize, buflen: usize, repeat: bool) -> StereoSampler<BUFSIZE> {
        StereoSampler {
            phase: 2, // start with one to account for interpolation samples on each side
            frac_phase: 2.0,
            bufnum,
            buflen, // length WITHOUT interpolation samples
            buflen_plus_one: buflen + 1,
            buflen_plus_one_f32: (buflen + 1) as f32,
            playback_rate: 1.0,
            frac_phase_increment: 1.0,
            state: SynthState::Fresh,
            amp: 1.0,
            repeat,
            rate_mod: None,
            amp_mod: None,
        }
    }

    fn get_next_block_plain(
        &mut self,
        start_sample: usize,
        sample_buffers: &[SampleBuffer],
    ) -> [[f32; BUFSIZE]; 2] {
        let mut out_buf: [[f32; BUFSIZE]; 2] = [[0.0; BUFSIZE]; 2];

        if let SampleBuffer::Stereo(left, right) = &sample_buffers[self.bufnum] {
            for s in start_sample..BUFSIZE {
                out_buf[0][s] = left[self.phase] * self.amp;
                out_buf[1][s] = right[self.phase] * self.amp;

                // include buflen idx as we start counting at 1 due to interpolation
                if self.phase < self.buflen_plus_one {
                    self.phase += 1;
                } else if self.repeat {
                    // start counting at two to account for interpolation samples
                    self.frac_phase = 2.0;
                    self.phase = 2;
                } else {
                    self.finish();
                }
            }
        }

        out_buf
    }

    fn get_next_block_plain_reverse(
        &mut self,
        start_sample: usize,
        sample_buffers: &[SampleBuffer],
    ) -> [[f32; BUFSIZE]; 2] {
        let mut out_buf: [[f32; BUFSIZE]; 2] = [[0.0; BUFSIZE]; 2];

        if let SampleBuffer::Stereo(left, right) = &sample_buffers[self.bufnum] {
            for s in start_sample..BUFSIZE {
                out_buf[0][s] = left[self.phase] * self.amp;
                out_buf[1][s] = right[self.phase] * self.amp;

                // include buflen idx as we start counting at 2 due to interpolation
                if self.phase > 2 {
                    self.phase -= 1;
                } else if self.repeat {
                    self.frac_phase = self.buflen_plus_one_f32;
                    self.phase = self.buflen_plus_one;
                } else {
                    self.finish();
                }
            }
        }

        out_buf
    }

    fn get_next_block_interpolated(
        &mut self,
        start_sample: usize,
        sample_buffers: &[SampleBuffer],
    ) -> [[f32; BUFSIZE]; 2] {
        let mut out_buf: [[f32; BUFSIZE]; 2] = [[0.0; BUFSIZE]; 2];

        if let SampleBuffer::Stereo(left, right) = &sample_buffers[self.bufnum] {
            for s in start_sample..BUFSIZE {
                // get sample:
                let idx = self.frac_phase.floor();
                let frac = self.frac_phase - idx;
                let idx_u = idx as usize;

                // 4-point, 3rd-order Hermite
                out_buf[0][s] = interpolate(
                    frac,
                    left[idx_u - 1],
                    left[idx_u],
                    left[idx_u + 1],
                    left[idx_u + 2],
                    self.amp,
                );

                // 4-point, 3rd-order Hermite
                out_buf[1][s] = interpolate(
                    frac,
                    right[idx_u - 1],
                    right[idx_u],
                    right[idx_u + 1],
                    right[idx_u + 2],
                    self.amp,
                );

                self.frac_phase += self.frac_phase_increment;

                // include buflen idx as we start counting at 1 due to interpolation
                if self.repeat && self.frac_phase.floor() > self.buflen_plus_one_f32 {
                    // again, start counting at two (at some point i should use the correct fraction here ...)
                    self.frac_phase = 2.0;
                    self.phase = 2;
                } else {
                    self.finish();
                }
            }
        }

        out_buf
    }

    fn get_next_block_interpolated_reverse(
        &mut self,
        start_sample: usize,
        sample_buffers: &[SampleBuffer],
    ) -> [[f32; BUFSIZE]; 2] {
        let mut out_buf: [[f32; BUFSIZE]; 2] = [[0.0; BUFSIZE]; 2];

        if let SampleBuffer::Stereo(left, right) = &sample_buffers[self.bufnum] {
            for s in start_sample..BUFSIZE {
                // get sample:
                let idx = self.frac_phase.ceil();
                let frac = idx - self.frac_phase;
                let idx_u = idx as usize;

                // 4-point, 3rd-order Hermite
                out_buf[0][s] = interpolate(
                    frac,
                    left[idx_u + 1],
                    left[idx_u],
                    left[idx_u - 1],
                    left[idx_u - 2],
                    self.amp,
                );

                // 4-point, 3rd-order Hermite
                out_buf[1][s] = interpolate(
                    frac,
                    right[idx_u + 1],
                    right[idx_u],
                    right[idx_u - 1],
                    right[idx_u - 2],
                    self.amp,
                );

                self.frac_phase += self.frac_phase_increment;

                // mind the buffer padding here ...
                if self.repeat && self.frac_phase.ceil() < 2.0 {
                    self.frac_phase = self.buflen_plus_one_f32;
                    self.phase = self.buflen_plus_one;
                } else {
                    self.finish();
                }
            }
        }

        out_buf
    }

    fn get_next_block_modulated(
        &mut self,
        start_sample: usize,
        sample_buffers: &[SampleBuffer],
    ) -> [[f32; BUFSIZE]; 2] {
        let mut out_buf: [[f32; BUFSIZE]; 2] = [[0.0; BUFSIZE]; 2];

        if let SampleBuffer::Stereo(left, right) = &sample_buffers[self.bufnum] {
            let rate_buf = if let Some(m) = self.rate_mod.as_mut() {
                m.process(self.playback_rate, start_sample, sample_buffers)
            } else {
                [self.playback_rate; BUFSIZE]
            };

            let amp_buf = if let Some(m) = self.amp_mod.as_mut() {
                m.process(self.amp, start_sample, sample_buffers)
            } else {
                [self.amp; BUFSIZE]
            };

            for sample_idx in start_sample..BUFSIZE {
                self.frac_phase_increment = rate_buf[sample_idx];

                if self.frac_phase_increment.is_sign_positive() {
                    // get sample:
                    let idx = self.frac_phase.floor();
                    let frac = self.frac_phase - idx;
                    let idx_u = idx as usize;

                    // 4-point, 3rd-order Hermite
                    out_buf[0][sample_idx] = interpolate(
                        frac,
                        left[idx_u - 1],
                        left[idx_u],
                        left[idx_u + 1],
                        left[idx_u + 2],
                        amp_buf[sample_idx],
                    );

                    out_buf[1][sample_idx] = interpolate(
                        frac,
                        right[idx_u - 1],
                        right[idx_u],
                        right[idx_u + 1],
                        right[idx_u + 2],
                        amp_buf[sample_idx],
                    );
                } else {
                    // get sample:
                    let idx = self.frac_phase.ceil();
                    let frac = idx - self.frac_phase;
                    let idx_u = idx as usize;

                    // 4-point, 3rd-order Hermite
                    out_buf[0][sample_idx] = interpolate(
                        frac,
                        left[idx_u + 1],
                        left[idx_u],
                        left[idx_u - 1],
                        left[idx_u - 2],
                        amp_buf[sample_idx],
                    );

                    out_buf[1][sample_idx] = interpolate(
                        frac,
                        right[idx_u + 1],
                        right[idx_u],
                        right[idx_u - 1],
                        right[idx_u - 2],
                        amp_buf[sample_idx],
                    );
                }

                self.frac_phase += self.frac_phase_increment;

                if self.repeat && self.frac_phase.floor() > self.buflen_plus_one_f32 {
                    self.frac_phase = 2.0;
                    self.phase = 2;
                } else if self.repeat && self.frac_phase.ceil() < 2.0 {
                    self.frac_phase = self.buflen_plus_one_f32;
                    self.phase = self.buflen_plus_one;
                } else {
                    self.finish();
                }
            }
        }
        out_buf
    }
}

impl<const BUFSIZE: usize> StereoSource<BUFSIZE> for StereoSampler<BUFSIZE> {
    fn reset(&mut self) {}

    fn set_modulator(
        &mut self,
        par: SynthParameterLabel,
        init: f32,
        modulator: Modulator<BUFSIZE>,
    ) {
        match par {
            SynthParameterLabel::PlaybackRate => {
                self.playback_rate = init;
                self.rate_mod = Some(modulator);
            }
            SynthParameterLabel::OscillatorAmplitude => {
                self.amp = init;
                self.amp_mod = Some(modulator);
            }
            _ => {}
        }
    }
    fn set_parameter(&mut self, par: SynthParameterLabel, val: &SynthParameterValue) {
        match par {
            SynthParameterLabel::PlaybackStart => {
                if let SynthParameterValue::ScalarF32(value_ref) = val {
                    let value = *value_ref;
                    let mut value_clamped = value;
                    // clamp value
                    if value == 1.0 {
                        value_clamped = 0.0
                    } else if value > 1.0 {
                        value_clamped = value - ((value as usize) as f32);
                    } else if value < 0.0 {
                        let v_abs = value.abs();
                        let v_abs_clamped = v_abs - ((v_abs as usize) as f32);
                        value_clamped = 1.0 - v_abs_clamped;
                    }

                    // as the start value is [0.0, 1.0), the offset will always be
                    // smaller than self.buflen ...
                    let offset = (self.buflen as f32 * value_clamped) as usize;
                    self.phase = offset + 2; // start counting at one, due to interpolation
                                             //println!("setting starting point to sample {}", self.phase);
                    self.frac_phase = self.phase as f32;
                }
            }
            SynthParameterLabel::PlaybackRate => {
                if let SynthParameterValue::ScalarF32(value) = val {
                    self.playback_rate = *value;
                    // I really don't know what the 1.0 is supposed to do here ...
                    // but by now I'm afraid to take it out ...
                    self.frac_phase_increment = 1.0 * *value;
                }
            }
            SynthParameterLabel::OscillatorAmplitude => {
                if let SynthParameterValue::ScalarF32(value) = val {
                    self.amp = *value;
                }
            }
            _ => (),
        };
    }

    fn finish(&mut self) {
        self.state = SynthState::Finished;
    }

    fn is_finished(&self) -> bool {
        matches!(self.state, SynthState::Finished)
    }

    fn get_next_block(
        &mut self,
        start_sample: usize,
        sample_buffers: &[SampleBuffer],
    ) -> [[f32; BUFSIZE]; 2] {
        if self.rate_mod.is_some() || self.amp_mod.is_some() {
            self.get_next_block_modulated(start_sample, sample_buffers)
        } else if self.playback_rate == 1.0 {
            self.get_next_block_plain(start_sample, sample_buffers)
        } else if self.playback_rate == -1.0 {
            self.get_next_block_plain_reverse(start_sample, sample_buffers)
        } else if self.playback_rate.is_sign_negative() {
            self.get_next_block_interpolated_reverse(start_sample, sample_buffers)
        } else {
            self.get_next_block_interpolated(start_sample, sample_buffers)
        }
    }
}
