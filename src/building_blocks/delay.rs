use crate::building_blocks::filters::*;
use crate::building_blocks::interpolation::*;
use crate::building_blocks::{
    Modulator, MonoEffect, SynthParameterLabel, SynthParameterValue, ValueOrModulator,
};

pub struct MonoDelay<const BUFSIZE: usize> {
    // user parameters
    rate: f32,
    time: f32,
    feedback: f32,

    // internal parameters
    buffer: Vec<f32>, // max 2 sec for now
    buffer_ptr: f32,
    max_buffer_ptr: f32,
    dampening_filter: Lpf18<BUFSIZE>,
    samplerate: f32,

    // modulator slots
    rate_mod: Option<Modulator<BUFSIZE>>,
    time_mod: Option<Modulator<BUFSIZE>>,
    fb_mod: Option<Modulator<BUFSIZE>>,
}

impl<const BUFSIZE: usize> MonoDelay<BUFSIZE> {
    pub fn new(sr: f32) -> Self {
        MonoDelay {
            rate: 1.0,
            time: 0.256,
            buffer: vec![0.0; sr as usize * 2 + 3],
            buffer_ptr: 1.0,
            max_buffer_ptr: (sr * 0.256) + 1.0, // 256 ms default time
            feedback: 0.5,
            dampening_filter: Lpf18::new(3000.0, 0.4, 0.3, 44100.0),
            samplerate: sr,
            rate_mod: None,
            time_mod: None,
            fb_mod: None,
        }
    }
}

impl<const BUFSIZE: usize> MonoEffect<BUFSIZE> for MonoDelay<BUFSIZE> {
    fn set_modulator(
        &mut self,
        par: SynthParameterLabel,
        init: f32,
        modulator: Modulator<BUFSIZE>,
    ) {
        match par {
            SynthParameterLabel::DelayDampeningFrequency => {
                self.dampening_filter.set_modulator(
                    SynthParameterLabel::LowpassCutoffFrequency,
                    init,
                    modulator,
                );
            }
            SynthParameterLabel::DelayFeedback => {
                self.feedback = init;
                self.fb_mod = Some(modulator);
            }
            SynthParameterLabel::DelayRate => {
                self.rate = init;
                self.rate_mod = Some(modulator);
            }
            SynthParameterLabel::DelayTime => {
                self.time = init;
                self.time_mod = Some(modulator);
            }
            _ => {}
        }
    }
    // some parameter limits might be nice ...
    fn set_parameter(&mut self, par: SynthParameterLabel, value: &SynthParameterValue) {
        if let SynthParameterValue::ScalarF32(val) = value {
            match par {
                SynthParameterLabel::DelayDampeningFrequency => self
                    .dampening_filter
                    .set_parameter(SynthParameterLabel::LowpassCutoffFrequency, value),
                SynthParameterLabel::DelayFeedback => self.feedback = *val,
                SynthParameterLabel::DelayRate => self.rate = *val,
                SynthParameterLabel::DelayTime => {
                    self.time = *val;
                    self.max_buffer_ptr = self.samplerate * self.time + 1.0;
                }
                _ => (),
            };
        }
    }

    fn finish(&mut self) {} // this effect is stateless
    fn is_finished(&self) -> bool {
        false
    } // it's never finished ..

    // start sample isn't really needed either ...
    fn process_block(
        &mut self,
        block: [f32; BUFSIZE],
        start_sample: usize,
        in_buffers: &[SampleBuffer],
    ) -> [f32; BUFSIZE] {
        let mut out_buf: [f32; BUFSIZE] = [0.0; BUFSIZE];

        if self.fb_mod.is_some() || self.rate_mod.is_some() || self.time_mod.is_some() {
            let fb_buf = if let Some(m) = self.fb_mod.as_mut() {
                m.process(self.feedback, start_sample, in_buffers)
            } else {
                [self.feedback; BUFSIZE]
            };

            let rate_buf = if let Some(m) = self.rate_mod.as_mut() {
                m.process(self.rate, start_sample, in_buffers)
            } else {
                [self.rate; BUFSIZE]
            };

            let time_buf = if let Some(m) = self.time_mod.as_mut() {
                m.process(self.time, start_sample, in_buffers)
                    .map(|x| (self.samplerate * x) + 1.0)
            } else {
                [self.samplerate * self.time; BUFSIZE]
            };

            for i in 0..BUFSIZE {
                // get sample:
                let idx = self.buffer_ptr.floor();
                let frac = self.buffer_ptr - idx;
                let idx_u = idx as usize;

                // 4-point, 3rd-order Hermite
                let buf_out = interpolate(
                    frac,
                    self.buffer[idx_u - 1],
                    self.buffer[idx_u],
                    self.buffer[idx_u + 1],
                    self.buffer[idx_u + 2],
                    1.0,
                );

                self.buffer[idx_u] =
                    (self.dampening_filter.process_sample(buf_out) * fb_buf[i]) + block[i];

                out_buf[i] = self.buffer[idx_u];

                // increment delay idx
                self.buffer_ptr += rate_buf[i];
                if self.buffer_ptr >= time_buf[i] {
                    self.buffer_ptr = 1.0 + (self.buffer_ptr - time_buf[i]);
                }
            }
        } else {
            for i in 0..BUFSIZE {
                // get sample:
                let idx = self.buffer_ptr.floor();
                let frac = self.buffer_ptr - idx;
                let idx_u = idx as usize;

                // 4-point, 3rd-order Hermite
                let buf_out = interpolate(
                    frac,
                    self.buffer[idx_u - 1],
                    self.buffer[idx_u],
                    self.buffer[idx_u + 1],
                    self.buffer[idx_u + 2],
                    1.0,
                );

                self.buffer[idx_u] =
                    (self.dampening_filter.process_sample(buf_out) * self.feedback) + block[i];

                out_buf[i] = self.buffer[idx_u];

                // increment delay idx
                self.buffer_ptr += self.rate;
                if self.buffer_ptr >= self.max_buffer_ptr {
                    self.buffer_ptr = 1.0 + (self.buffer_ptr - self.max_buffer_ptr);
                }
            }
        }

        out_buf
    }
}

pub struct MultichannelDelay<const BUFSIZE: usize, const NCHAN: usize> {
    delays: Vec<MonoDelay<BUFSIZE>>,
}

impl<const BUFSIZE: usize, const NCHAN: usize> MultichannelDelay<BUFSIZE, NCHAN> {
    pub fn new(sr: f32) -> Self {
        let mut delays = Vec::new();

        for _ in 0..NCHAN {
            delays.push(MonoDelay::<BUFSIZE>::new(sr));
        }

        MultichannelDelay { delays }
    }

    pub fn set_parameter(&mut self, par: SynthParameterLabel, val: &SynthParameterValue) {
        for c in 0..NCHAN {
            self.delays[c].set_parameter(par, val);
        }
    }

    pub fn set_param_or_modulator(
        &mut self,
        par: SynthParameterLabel,
        val_or_mod: ValueOrModulator<BUFSIZE>,
    ) {
        match val_or_mod {
            ValueOrModulator::Val(val) => self.set_parameter(par, &val),
            ValueOrModulator::Mod(init, modulator) => self.set_modulator(par, init, modulator),
        }
    }

    pub fn set_modulator(
        &mut self,
        par: SynthParameterLabel,
        init: f32,
        modulator: Modulator<BUFSIZE>,
    ) {
        for c in 0..NCHAN {
            // i might want to think of a better method because
            // this clone will be called in the audio thread, even though
            // this won't be super common, but still ...
            self.delays[c].set_modulator(par, init, modulator.clone());
        }
    }

    pub fn process(
        &mut self,
        block: [[f32; BUFSIZE]; NCHAN],
        sample_buffers: &[SampleBuffer],
    ) -> [[f32; BUFSIZE]; NCHAN] {
        let mut out_buf = [[0.0; BUFSIZE]; NCHAN];

        for c in 0..NCHAN {
            out_buf[c] = self.delays[c].process_block(block[c], 0, sample_buffers);
        }

        out_buf
    }
}
