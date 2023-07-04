// parent imports
use crate::building_blocks::{
    interpolation::*, Modulator, MonoSource, SampleBuffer, SynthParameterLabel,
    SynthParameterValue, SynthState,
};

/**
 * a very simple sample player ...
 */
#[derive(Clone)]
pub struct MonoSampler<const BUFSIZE: usize> {
    // user parameters
    playback_rate: f32,
    amp: f32,

    // internal parameters
    index: usize,
    frac_index: f32,
    bufnum: usize,
    buflen: usize,
    frac_index_increment: f32,
    state: SynthState,
    repeat: bool,

    // modulator slots
    rate_mod: Option<Modulator<BUFSIZE>>,
    amp_mod: Option<Modulator<BUFSIZE>>,
}

impl<const BUFSIZE: usize> MonoSampler<BUFSIZE> {
    pub fn with_bufnum_len(bufnum: usize, buflen: usize, repeat: bool) -> MonoSampler<BUFSIZE> {
        MonoSampler {
            index: 1, // start with one to account for interpolation
            frac_index: 1.0,
            bufnum,
            buflen,
            playback_rate: 1.0,
            frac_index_increment: 1.0,
            state: SynthState::Fresh,
            amp: 1.0,
            repeat,
            rate_mod: None,
            amp_mod: None,
        }
    }

    // standard speed playback, no interpolation needed ...
    fn get_next_block_plain(
        &mut self,
        start_sample: usize,
        sample_buffers: &[SampleBuffer],
    ) -> [f32; BUFSIZE] {
        let mut out_buf: [f32; BUFSIZE] = [0.0; BUFSIZE];
        if let SampleBuffer::Mono(buf) = &sample_buffers[self.bufnum] {
            for current_sample in out_buf.iter_mut().take(BUFSIZE).skip(start_sample) {
                *current_sample = buf[self.index] * self.amp;

                // include buflen idx as we start counting at 1 due to interpolation
                if self.index < self.buflen {
                    self.index += 1;
                } else if self.repeat {
                    self.frac_index = 1.0;
                    self.index = 1;
                } else {
                    self.finish();
                }
            }
        }

        out_buf
    }

    // reverse standard speed playback, no interpolation needed ...
    fn get_next_block_plain_reverse(
        &mut self,
        start_sample: usize,
        sample_buffers: &[SampleBuffer],
    ) -> [f32; BUFSIZE] {
        let mut out_buf: [f32; BUFSIZE] = [0.0; BUFSIZE];
        if let SampleBuffer::Mono(buf) = &sample_buffers[self.bufnum] {
            for current_sample in out_buf.iter_mut().take(BUFSIZE).skip(start_sample) {
                *current_sample = buf[self.index] * self.amp;

                // include buflen idx as we start counting at 1 due to interpolation
                if self.index > 1 {
                    self.index -= 1;
                } else if self.repeat {
                    self.frac_index = (self.buflen - 1) as f32;
                    self.index = self.buflen - 1;
                } else {
                    self.finish();
                }
            }
        }

        out_buf
    }

    // positive rate other than 1.0
    fn get_next_block_interpolated(
        &mut self,
        start_sample: usize,
        sample_buffers: &[SampleBuffer],
    ) -> [f32; BUFSIZE] {
        let mut out_buf: [f32; BUFSIZE] = [0.0; BUFSIZE];
        if let SampleBuffer::Mono(buf) = &sample_buffers[self.bufnum] {
            for current_sample in out_buf.iter_mut().take(BUFSIZE).skip(start_sample) {
                // get sample:
                let idx = self.frac_index.floor();
                let frac = self.frac_index - idx;
                let idx_u = idx as usize;

                // 4-point, 3rd-order Hermite
                *current_sample = interpolate(
                    frac,
                    buf[idx_u - 1],
                    buf[idx_u],
                    buf[idx_u + 1],
                    buf[idx_u + 2],
                    self.amp,
                );

                // include buflen idx as we start counting at 1 due to interpolation
                if ((self.frac_index + self.frac_index_increment) as usize) < self.buflen {
                    self.frac_index += self.frac_index_increment;
                } else if self.repeat {
                    self.frac_index = 1.0;
                    self.index = 1;
                } else {
                    self.finish();
                }
            }
        }
        out_buf
    }

    // negative rate other than -1.0
    fn get_next_block_interpolated_reverse(
        &mut self,
        start_sample: usize,
        sample_buffers: &[SampleBuffer],
    ) -> [f32; BUFSIZE] {
        let mut out_buf: [f32; BUFSIZE] = [0.0; BUFSIZE];
        if let SampleBuffer::Mono(buf) = &sample_buffers[self.bufnum] {
            for current_sample in out_buf.iter_mut().take(BUFSIZE).skip(start_sample) {
                // get sample:
                let idx = self.frac_index.ceil();
                let frac = (self.frac_index - idx).abs();
                let idx_u = idx as usize;

                // aaargh ...
                if idx_u >= 2 {
                    // 4-point, 3rd-order Hermite
                    *current_sample = interpolate(
                        frac,
                        buf[idx_u + 1],
                        buf[idx_u],
                        buf[idx_u - 1],
                        buf[idx_u - 2],
                        self.amp,
                    );
                }

                self.frac_index += self.frac_index_increment;

                // mind the buffer padding here ...
                if self.repeat && self.frac_index < 2.0 {
                    self.frac_index = (self.buflen - 2) as f32;
                    self.index = self.buflen - 2;
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
    ) -> [f32; BUFSIZE] {
        let mut out_buf: [f32; BUFSIZE] = [0.0; BUFSIZE];

        // this is a mono-only sampler
        if let SampleBuffer::Mono(buf) = &sample_buffers[self.bufnum] {
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

            for (sample_idx, current_sample) in out_buf
                .iter_mut()
                .enumerate()
                .take(BUFSIZE)
                .skip(start_sample)
            {
                // again, no idea what the 1.0 is about, but too afraid to remove ...
                // leave it here for good luck ...
                self.frac_index_increment = 1.0 * rate_buf[sample_idx];

                if self.frac_index_increment.is_sign_positive() {
                    // get sample:
                    let idx = self.frac_index.floor();
                    let frac = self.frac_index - idx;
                    let idx_u = idx as usize;

                    // 4-point, 3rd-order Hermite
                    *current_sample = interpolate(
                        frac,
                        buf[idx_u - 1],
                        buf[idx_u],
                        buf[idx_u + 1],
                        buf[idx_u + 2],
                        amp_buf[sample_idx],
                    );
                } else {
                    // get sample:
                    let idx = self.frac_index.ceil();
                    let frac = (self.frac_index - idx).abs();
                    let idx_u = idx as usize;

                    if idx_u > 2 {
                        // 4-point, 3rd-order Hermite
                        *current_sample = interpolate(
                            frac,
                            buf[idx_u + 1],
                            buf[idx_u],
                            buf[idx_u - 1],
                            buf[idx_u - 2],
                            amp_buf[sample_idx],
                        );
                    }
                }

                self.frac_index += self.frac_index_increment;

                if self.repeat && self.frac_index >= self.buflen as f32 {
                    self.frac_index = 1.0;
                    self.index = 1;
                } else if self.repeat && self.frac_index < 2.0 {
                    self.frac_index = (self.buflen - 1) as f32;
                    self.index = self.buflen - 1;
                } else {
                    self.finish();
                }
            }
        }

        out_buf
    }
}

impl<const BUFSIZE: usize> MonoSource<BUFSIZE> for MonoSampler<BUFSIZE> {
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
                    self.index = offset + 1; // start counting at one, due to interpolation

                    self.frac_index = self.index as f32;
                }
            }
            SynthParameterLabel::PlaybackRate => {
                if let SynthParameterValue::ScalarF32(value) = val {
                    self.playback_rate = *value;
                    // I really don't know what the 1.0 is supposed to do here ...
                    // but by now I'm afraid to take it out ...
                    self.frac_index_increment = 1.0 * *value;
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
    ) -> [f32; BUFSIZE] {
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
