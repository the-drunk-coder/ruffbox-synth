use crate::building_blocks::{Modulator, MonoSource, SynthParameterLabel, SynthParameterValue};

use std::f32::consts::PI;

/**
 * A simple sine oscillator
 */
pub struct SineOsc<const BUFSIZE: usize> {
    lvl: f32,
    samplerate: f32,
    //delta_t: f32,
    freq: f32,
    x1_last: f32,
    x2_last: f32,
    mcf: f32,
    freq_mod: Option<Modulator<BUFSIZE>>,
    lvl_mod: Option<Modulator<BUFSIZE>>,
}

impl<const BUFSIZE: usize> SineOsc<BUFSIZE> {
    pub fn new(freq: f32, lvl: f32, sr: f32) -> Self {
        SineOsc {
            lvl,
            //delta_t: 1.0 / sr,
            samplerate: sr,
            x1_last: 0.0,
            x2_last: 1.0,
            mcf: 2.0 * (PI * freq * 1.0 / sr).sin(),
            freq,
            freq_mod: None,
            lvl_mod: None,
        }
    }
}

impl<const BUFSIZE: usize> MonoSource<BUFSIZE> for SineOsc<BUFSIZE> {
    // some parameter limits might be nice ...
    fn set_parameter(&mut self, par: SynthParameterLabel, value: &SynthParameterValue) {
        match par {
            SynthParameterLabel::PitchFrequency => {
                match value {
                    SynthParameterValue::ScalarF32(f) => {
                        self.freq = *f;
                        self.mcf = 2.0 * (PI * f * 1.0 / self.samplerate).sin()
                    }
                    SynthParameterValue::Lfo(freq, range, op) => {
                        self.freq_mod = Some(Modulator::lfo(*op, *freq, *range, self.samplerate))
                    }
                    _ => { /* nothing to do, don't know how to handle this ... */ }
                }
            }
            SynthParameterLabel::Level => {
                match value {
                    SynthParameterValue::ScalarF32(l) => {
                        self.lvl = *l;
                    }
                    SynthParameterValue::Lfo(freq, range, op) => {
                        self.lvl_mod = Some(Modulator::lfo(*op, *freq, *range, self.samplerate))
                    }
                    _ => { /* nothing to do, don't know how to handle this ... */ }
                }
            }
            _ => (),
        };
    }

    fn finish(&mut self) {
        //self.state = SynthState::Finished;
    }

    fn is_finished(&self) -> bool {
        false
    }

    fn get_next_block(&mut self, start_sample: usize, _in_buffers: &[Vec<f32>]) -> [f32; BUFSIZE] {
        let mut out_buf: [f32; BUFSIZE] = [0.0; BUFSIZE];

        for current_sample in out_buf.iter_mut().take(BUFSIZE).skip(start_sample) {
            let x1 = self.x1_last + (self.mcf * self.x2_last);
            let x2 = -self.mcf * x1 + self.x2_last;
            *current_sample = x2 * self.lvl;
            self.x1_last = x1;
            self.x2_last = x2;
        }

        out_buf
    }
}
