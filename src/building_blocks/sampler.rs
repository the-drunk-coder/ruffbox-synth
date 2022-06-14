// parent imports
use crate::building_blocks::{
    interpolation::*, Modulator, MonoSource, SynthParameterLabel, SynthParameterValue, SynthState,
    ValueOrModulator,
};

/**
 * a very simple sample player ...
 */
#[derive(Clone)]
pub struct Sampler<const BUFSIZE: usize> {
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

impl<const BUFSIZE: usize> Sampler<BUFSIZE> {
    pub fn with_bufnum_len(bufnum: usize, buflen: usize, repeat: bool) -> Sampler<BUFSIZE> {
        Sampler {
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

    fn get_next_block_plain(
        &mut self,
        start_sample: usize,
        sample_buffers: &[Vec<f32>],
    ) -> [f32; BUFSIZE] {
        let mut out_buf: [f32; BUFSIZE] = [0.0; BUFSIZE];

        for current_sample in out_buf.iter_mut().take(BUFSIZE).skip(start_sample) {
            *current_sample = sample_buffers[self.bufnum][self.index] * self.amp;

            if self.index < self.buflen {
                self.index += 1;
            } else if self.repeat {
                self.frac_index = 1.0;
                self.index = 1;
            } else {
                self.finish();
            }
        }

        out_buf
    }

    fn get_next_block_interpolated(
        &mut self,
        start_sample: usize,
        sample_buffers: &[Vec<f32>],
    ) -> [f32; BUFSIZE] {
        let mut out_buf: [f32; BUFSIZE] = [0.0; BUFSIZE];

        for current_sample in out_buf.iter_mut().take(BUFSIZE).skip(start_sample) {
            // get sample:
            let idx = self.frac_index.floor();
            let frac = self.frac_index - idx;
            let idx_u = idx as usize;

            // 4-point, 3rd-order Hermite
            *current_sample = interpolate(
                frac,
                sample_buffers[self.bufnum][idx_u - 1],
                sample_buffers[self.bufnum][idx_u],
                sample_buffers[self.bufnum][idx_u + 1],
                sample_buffers[self.bufnum][idx_u + 2],
                self.amp,
            );

            if ((self.frac_index + self.frac_index_increment) as usize) < self.buflen {
                self.frac_index += self.frac_index_increment;
            } else if self.repeat {
                self.frac_index = 1.0;
                self.index = 1;
            } else {
                self.finish();
            }
        }

        out_buf
    }

    fn get_next_block_modulated(
        &mut self,
        start_sample: usize,
        sample_buffers: &[Vec<f32>],
    ) -> [f32; BUFSIZE] {
        let mut out_buf: [f32; BUFSIZE] = [0.0; BUFSIZE];

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
            self.frac_index_increment = 1.0 * rate_buf[sample_idx];

            // get sample:
            let idx = self.frac_index.floor();
            let frac = self.frac_index - idx;
            let idx_u = idx as usize;

            // 4-point, 3rd-order Hermite
            *current_sample = interpolate(
                frac,
                sample_buffers[self.bufnum][idx_u - 1],
                sample_buffers[self.bufnum][idx_u],
                sample_buffers[self.bufnum][idx_u + 1],
                sample_buffers[self.bufnum][idx_u + 2],
                amp_buf[sample_idx],
            );

            if ((self.frac_index + self.frac_index_increment) as usize) < self.buflen {
                self.frac_index += self.frac_index_increment;
            } else if self.repeat {
                self.frac_index = 1.0;
                self.index = 1;
            } else {
                self.finish();
            }
        }

        out_buf
    }
}

impl<const BUFSIZE: usize> MonoSource<BUFSIZE> for Sampler<BUFSIZE> {
    fn reset(&mut self) {}

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
                    if value > 1.0 {
                        value_clamped = value - ((value as usize) as f32);
                    } else if value < 0.0 {
                        value_clamped = 1.0 + (value - ((value as i32) as f32));
                    }

                    let offset = ((self.buflen - 1) as f32 * value_clamped) as usize;
                    self.index = offset + 1; // start counting at one, due to interpolation
                                             //println!("setting starting point to sample {}", self.index);
                    self.frac_index = self.index as f32;
                }
            }
            SynthParameterLabel::PlaybackRate => {
                if let SynthParameterValue::ScalarF32(value) = val {
                    self.playback_rate = *value;
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
        sample_buffers: &[Vec<f32>],
    ) -> [f32; BUFSIZE] {
        if self.rate_mod.is_some() || self.rate_mod.is_some() {
            self.get_next_block_modulated(start_sample, sample_buffers)
        } else if self.playback_rate == 1.0 {
            self.get_next_block_plain(start_sample, sample_buffers)
        } else {
            self.get_next_block_interpolated(start_sample, sample_buffers)
        }
    }
}
