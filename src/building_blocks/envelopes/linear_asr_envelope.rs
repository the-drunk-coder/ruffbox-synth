use crate::building_blocks::{
    Modulator, MonoEffect, SynthParameterLabel, SynthParameterValue, SynthState,
};

/// simple linear attack-sustain-release envelope
#[derive(Clone, Copy)]
pub struct LinearASREnvelope<const BUFSIZE: usize> {
    samplerate: f32,
    atk: f32,
    sus: f32,
    rel: f32,
    atk_samples: usize,
    sus_samples: usize,
    rel_samples: usize,
    sample_count: usize,
    lvl: f32,
    max_lvl: f32,
    atk_lvl_increment: f32,
    rel_lvl_decrement: f32,
    state: SynthState,
}

impl<const BUFSIZE: usize> LinearASREnvelope<BUFSIZE> {
    pub fn new(lvl: f32, atk: f32, sus: f32, rel: f32, samplerate: f32) -> Self {
        let atk_samples = (samplerate * atk).round();
        let sus_samples = atk_samples + (samplerate * sus).round();
        let rel_samples = sus_samples + (samplerate * rel).round();

        LinearASREnvelope {
            samplerate,
            atk,
            sus,
            rel,
            atk_samples: atk_samples as usize,
            sus_samples: sus_samples as usize,
            rel_samples: rel_samples as usize,
            sample_count: 0,
            lvl: 0.0,
            max_lvl: lvl,
            atk_lvl_increment: lvl / atk_samples,
            rel_lvl_decrement: lvl / (rel_samples - sus_samples),
            state: SynthState::Fresh,
        }
    }
}

impl<const BUFSIZE: usize> MonoEffect<BUFSIZE> for LinearASREnvelope<BUFSIZE> {
    fn finish(&mut self) {
        self.state = SynthState::Finished;
    }

    fn is_finished(&self) -> bool {
        matches!(self.state, SynthState::Finished)
    }
   
    fn set_modulator(&mut self, _: SynthParameterLabel, _: f32, _: Modulator<BUFSIZE>) {}

    fn set_parameter(&mut self, par: SynthParameterLabel, value: &SynthParameterValue) {
        let mut update_internals = false;

        match value {
            SynthParameterValue::ScalarF32(val) => {
                match par {
                    // leave those here for legacy reasons ...
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
                    SynthParameterLabel::EnvelopeLevel => {
                        self.max_lvl = *val;
                        update_internals = true;
                    }
                    SynthParameterLabel::Samplerate => {
                        self.samplerate = *val;
                        update_internals = true;
                    }

                    _ => (),
                }
            }

            // this is the one that should be used ...
            SynthParameterValue::MultiPointEnvelope(segments, _, _) => {
                println!("segments {}", segments.len());
                if segments.len() == 3 {
                    // ASR
                    self.atk = segments[0].time;
                    self.sus = segments[1].time;
                    self.rel = segments[2].time;
                    self.max_lvl = segments[1].from;
                    update_internals = true;
                } else if segments.len() == 4 {
                    // ADSR, ignore D
                    self.atk = segments[0].time;
                    self.sus = segments[2].time;
                    self.rel = segments[3].time;
                    self.max_lvl = segments[2].from;
                    update_internals = true;
                } else {
                    // ignore ?
                }
            }
            _ => (),
        }

        if update_internals {
            self.atk_samples = (self.samplerate * self.atk).round() as usize;
            self.sus_samples = self.atk_samples + (self.samplerate * self.sus).round() as usize;
            self.rel_samples = self.sus_samples + (self.samplerate * self.rel).round() as usize;

            // keep values sane
            self.atk_lvl_increment = self.max_lvl / self.atk_samples as f32;
            if self.atk_lvl_increment != 0.0 && !self.atk_lvl_increment.is_normal() {
                self.atk_lvl_increment = 0.0;
            }

            self.rel_lvl_decrement = self.max_lvl / (self.rel_samples - self.sus_samples) as f32;
            if self.rel_lvl_decrement != 0.0 && !self.rel_lvl_decrement.is_normal() {
                self.rel_lvl_decrement = 0.0;
            }
        }
    }

    fn process_block(
        &mut self,
        block: [f32; BUFSIZE],
        start_sample: usize,
        _: &[Vec<f32>],
    ) -> [f32; BUFSIZE] {
        let mut out: [f32; BUFSIZE] = [0.0; BUFSIZE];

        for i in start_sample..BUFSIZE {
            out[i] = block[i] * self.lvl;

            self.sample_count += 1;
            if self.sample_count < self.atk_samples {
                self.lvl += self.atk_lvl_increment;
            } else if self.sample_count >= self.atk_samples && self.sample_count < self.sus_samples
            {
                self.lvl = self.max_lvl;
            } else if self.sample_count >= self.sus_samples
                && self.sample_count < self.rel_samples - 1
            {
                self.lvl -= self.rel_lvl_decrement;
            } else if self.sample_count >= self.rel_samples - 1 {
                self.lvl = 0.0;
                self.finish();
            }
        }
        out
    }
}
