use crate::building_blocks::filters::*;
use crate::building_blocks::{MonoEffect, SynthParameterLabel, SynthParameterValue};

pub struct MonoDelay<const BUFSIZE: usize> {
    buffer: Vec<f32>, // max 2 sec for now
    buffer_idx: usize,
    max_buffer_idx: usize,
    feedback: f32,
    dampening_filter: Lpf18<BUFSIZE>,
    samplerate: f32,
}

impl<const BUFSIZE: usize> MonoDelay<BUFSIZE> {
    pub fn new(sr: f32) -> Self {
        MonoDelay {
            buffer: vec![0.0; sr as usize * 2],
            buffer_idx: 0,
            max_buffer_idx: (sr * 0.256) as usize, // 512ms default time
            feedback: 0.5,
            dampening_filter: Lpf18::new(3000.0, 0.4, 0.3, 44100.0),
            samplerate: sr,
        }
    }
}

impl<const BUFSIZE: usize> MonoEffect<BUFSIZE> for MonoDelay<BUFSIZE> {
    // some parameter limits might be nice ...
    fn set_parameter(&mut self, par: SynthParameterLabel, value: &SynthParameterValue) {
        if let SynthParameterValue::ScalarF32(val) = value {
            match par {
                SynthParameterLabel::DelayDampeningFrequency => self
                    .dampening_filter
                    .set_parameter(SynthParameterLabel::LowpassCutoffFrequency, value),
                SynthParameterLabel::DelayFeedback => self.feedback = *val,
                SynthParameterLabel::DelayTime => {
                    self.max_buffer_idx = (self.samplerate * val) as usize
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
    fn process_block(&mut self, block: [f32; BUFSIZE], _: usize) -> [f32; BUFSIZE] {
        let mut out_buf: [f32; BUFSIZE] = [0.0; BUFSIZE];

        for i in 0..BUFSIZE {
            let buf_out = self.buffer[self.buffer_idx];

            self.buffer[self.buffer_idx] =
                (self.dampening_filter.process_sample(buf_out) * self.feedback) + block[i];

            out_buf[i] = self.buffer[self.buffer_idx];

            // increment delay idx
            self.buffer_idx += 1;
            if self.buffer_idx >= self.max_buffer_idx {
                self.buffer_idx = 0;
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

    pub fn process(&mut self, block: [[f32; BUFSIZE]; NCHAN]) -> [[f32; BUFSIZE]; NCHAN] {
        let mut out_buf = [[0.0; BUFSIZE]; NCHAN];

        for c in 0..NCHAN {
            out_buf[c] = self.delays[c].process_block(block[c], 0);
        }

        out_buf
    }
}
