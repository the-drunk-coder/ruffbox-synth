use crate::ruffbox::synth::MonoSource;
use crate::ruffbox::synth::SynthState;
use crate::ruffbox::synth::{SynthParameterLabel, SynthParameterValue};

/**
 * A simple, raw wavetable oscillator
 */
#[derive(Clone, Copy)]
pub struct Wavetable<const BUFSIZE: usize> {
    wavetable: [f32; 2048], // max len
    tablesize: usize,
    phase_inc: f32,
    table_ptr: f32,
    state: SynthState,
    level: f32,
    sample_period: f32,
}

impl<const BUFSIZE: usize> Wavetable<BUFSIZE> {
    pub fn new(sr: f32) -> Wavetable<BUFSIZE> {
        let sample_period = 1.0 / sr;
        Wavetable {
            wavetable: [0.0; 2048],
            tablesize: 2048,
            phase_inc: 1.0,
            table_ptr: 0.0,
            state: SynthState::Fresh,
            level: 1.0,
            sample_period,
        }
    }
}

impl<const BUFSIZE: usize> MonoSource<BUFSIZE> for Wavetable<BUFSIZE> {
    fn set_parameter(&mut self, par: SynthParameterLabel, val: &SynthParameterValue) {
        match par {
            SynthParameterLabel::PitchFrequency => {
                if let SynthParameterValue::ScalarF32(value) = val {
                    self.phase_inc = self.tablesize as f32 * *value * self.sample_period;
                }
            }
            SynthParameterLabel::Level => {
                if let SynthParameterValue::ScalarF32(value) = val {
                    self.level = *value;
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

    fn get_next_block(&mut self, start_sample: usize, _: &[Vec<f32>]) -> [f32; BUFSIZE] {
        let mut out_buf: [f32; BUFSIZE] = [0.0; BUFSIZE];

        for current_sample in out_buf.iter_mut().take(BUFSIZE).skip(start_sample) {
            // get sample:
            let idx = self.table_ptr as usize;
            let frac = self.table_ptr - (idx as f32);

            // use simple linear interpolation for now ...
            *current_sample = if frac == 0.0 {
                self.wavetable[idx]
            } else {
                let next_idx = if idx < self.tablesize - 1 { idx + 1 } else { 0 };
                self.wavetable[idx] + (frac * (self.wavetable[next_idx] - self.wavetable[idx]))
            };

            self.table_ptr += self.phase_inc;
        }

        out_buf
    }
}
