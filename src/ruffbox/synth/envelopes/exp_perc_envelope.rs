use crate::ruffbox::synth::MonoEffect;
use crate::ruffbox::synth::SynthState;
use crate::ruffbox::synth::{SynthParameterLabel, SynthParameterValue};

/// Exponential/Linear Percussion Envelope (currently with fixed curve value)
#[derive(Clone, Copy)]
pub struct ExpPercEnvelope<const BUFSIZE: usize> {
    samplerate: f32,
    atk: f32,
    sus: f32,
    rel: f32,
    atk_samples: usize,
    sus_samples: usize,
    rel_samples: usize,
    atk_inc: f32,
    rel_inc: f32,
    time_count: f32,
    curve: f32,
    sample_count: usize,
    max_lvl: f32,
    state: SynthState,
}

impl<const BUFSIZE: usize> ExpPercEnvelope<BUFSIZE> {
    pub fn new(lvl: f32, atk: f32, sus: f32, rel: f32, samplerate: f32) -> Self {
        let atk_samples = (samplerate * atk).round() as usize;
        let sus_samples = atk_samples + (samplerate * sus).round() as usize;
        let rel_samples = (samplerate * rel).round() as usize;

        let atk_inc = 1.0 / atk_samples as f32;
        let rel_inc = 1.0 / rel_samples as f32;

        //println!("atk sam: {} sus sam: {} rel sam: {}", atk_samples, sus_samples, rel_samples );

        ExpPercEnvelope {
            samplerate,
            atk,
            sus,
            rel,
            atk_samples,
            sus_samples,
            rel_samples: sus_samples + rel_samples,
            sample_count: 0,
            atk_inc,
            rel_inc,
            curve: -4.5,
            time_count: 0.0,
            max_lvl: lvl,
            state: SynthState::Fresh,
        }
    }
}

impl<const BUFSIZE: usize> MonoEffect<BUFSIZE> for ExpPercEnvelope<BUFSIZE> {
    fn finish(&mut self) {
        self.state = SynthState::Finished;
    }

    fn is_finished(&self) -> bool {
        matches!(self.state, SynthState::Finished)
    }

    fn set_parameter(&mut self, par: SynthParameterLabel, value: &SynthParameterValue) {
        let mut update_internals = false;
        if let SynthParameterValue::ScalarF32(val) = value {
            match par {
                SynthParameterLabel::Attack => {
                    self.atk = *val;
                    update_internals = true;
                }
                SynthParameterLabel::Sustain => {
                    self.sus = *val;
                    update_internals = true;
                }
                SynthParameterLabel::Release => {
                    self.rel = *val;
                    update_internals = true;
                }
                SynthParameterLabel::Level => {
                    self.max_lvl = *val;
                }
                SynthParameterLabel::Samplerate => {
                    self.samplerate = *val;
                    update_internals = true;
                }
                _ => (),
            };
        }
        if update_internals {
            self.atk_samples = (self.samplerate * self.atk).round() as usize;
            self.sus_samples = self.atk_samples + (self.samplerate * self.sus).round() as usize;
            self.rel_samples = (self.samplerate * self.rel).round() as usize;

            self.atk_inc = 1.0 / self.atk_samples as f32;
            self.rel_inc = 1.0 / self.rel_samples as f32;

            self.rel_samples += self.sus_samples;
        }
    }

    fn process_block(&mut self, block: [f32; BUFSIZE], start_sample: usize) -> [f32; BUFSIZE] {
        let mut out: [f32; BUFSIZE] = [0.0; BUFSIZE];

        for i in start_sample..BUFSIZE {
            let env = if self.sample_count < self.atk_samples {
                let env = ((self.curve * self.time_count).exp() - 1.0) / (self.curve.exp() - 1.0);
                self.time_count += self.atk_inc;
                env
            } else if self.sample_count >= self.atk_samples && self.sample_count <= self.sus_samples
            {
                self.time_count = 0.0; // this is a bit redundant ...
                1.0
            } else if self.sample_count > self.sus_samples
                && self.sample_count < self.rel_samples - 1
            {
                let env = ((self.curve * self.time_count).exp() - 1.0) / (self.curve.exp() - 1.0);
                self.time_count += self.rel_inc;
                1.0 - env
            } else {
                self.finish();
                0.0
            };

            out[i] = block[i] * env * self.max_lvl;

            self.sample_count += 1;
        }

        out
    }
}
