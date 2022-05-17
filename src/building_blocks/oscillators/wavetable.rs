use crate::building_blocks::{
    Modulator, MonoSource, SynthParameterLabel, SynthParameterValue, SynthState,
};

/**
 * A simple, raw wavetable oscillator
 */
pub struct Wavetable<const BUFSIZE: usize> {
    // user parameters
    lvl: f32,
    freq: f32,
    wavetable: [f32; 2048], // max len

    // internal parameters
    tablesize: usize,
    phase_inc: f32,
    table_ptr: f32,
    state: SynthState,
    sample_period: f32,
    samplerate: f32,

    // modulator slots
    freq_mod: Option<Modulator<BUFSIZE>>,
    lvl_mod: Option<Modulator<BUFSIZE>>,
}

impl<const BUFSIZE: usize> Wavetable<BUFSIZE> {
    pub fn new(sr: f32) -> Wavetable<BUFSIZE> {
        Wavetable {
            freq: 46.875,
            lvl: 1.0,
            wavetable: [0.5; 2048],
            tablesize: 2048,
            phase_inc: 1.0,
            table_ptr: 0.0,
            state: SynthState::Fresh,
            sample_period: 1.0 / sr,
            samplerate: sr,
            freq_mod: None,
            lvl_mod: None,
        }
    }
}

impl<const BUFSIZE: usize> MonoSource<BUFSIZE> for Wavetable<BUFSIZE> {
    fn set_parameter(&mut self, par: SynthParameterLabel, val: &SynthParameterValue) {
        match par {
            SynthParameterLabel::PitchFrequency => match val {
                SynthParameterValue::ScalarF32(value) => {
                    self.freq = *value;
                    self.phase_inc = self.tablesize as f32 * self.freq * self.sample_period;
                }
                SynthParameterValue::Lfo(init, freq, range, op) => {
                    self.freq = *init;
                    self.freq_mod = Some(Modulator::lfo(*op, *freq, *range, self.samplerate))
                }
                _ => {}
            },
            SynthParameterLabel::Wavetable => {
                if let SynthParameterValue::VecF32(tab) = val {
                    self.tablesize = std::cmp::min(tab.len(), 2048);
                    self.wavetable[..self.tablesize].copy_from_slice(&tab[..self.tablesize]);
                    self.phase_inc = self.tablesize as f32 * self.freq * self.sample_period;
                }
            }
            SynthParameterLabel::OscillatorLevel => match val {
                SynthParameterValue::ScalarF32(value) => {
                    self.lvl = *value;
                }
                SynthParameterValue::Lfo(init, freq, range, op) => {
                    self.lvl = *init;
                    self.lvl_mod = Some(Modulator::lfo(*op, *freq, *range, self.samplerate))
                }
                _ => {}
            },
            _ => (),
        };
    }

    fn finish(&mut self) {
        self.state = SynthState::Finished;
    }

    fn is_finished(&self) -> bool {
        matches!(self.state, SynthState::Finished)
    }

    fn get_next_block(&mut self, start_sample: usize, in_buffers: &[Vec<f32>]) -> [f32; BUFSIZE] {
        let mut out_buf: [f32; BUFSIZE] = [0.0; BUFSIZE];

        if self.freq_mod.is_some() || self.lvl_mod.is_some() {
            let lvl_buf = if let Some(m) = self.lvl_mod.as_mut() {
                m.process(self.lvl, start_sample, in_buffers)
            } else {
                [self.lvl; BUFSIZE]
            };

            let freq_buf = if let Some(m) = self.freq_mod.as_mut() {
                m.process(self.freq, start_sample, in_buffers)
            } else {
                [self.freq; BUFSIZE]
            };

            for (sample_idx, current_sample) in out_buf
                .iter_mut()
                .enumerate()
                .take(BUFSIZE)
                .skip(start_sample)
            {
                self.phase_inc = self.tablesize as f32 * freq_buf[sample_idx] * self.sample_period;

                // get sample:
                let idx = self.table_ptr as usize;
                let frac = self.table_ptr - (idx as f32);

                // use simple linear interpolation for now ...
                *current_sample = if frac == 0.0 {
                    self.wavetable[idx]
                } else {
                    let next_idx = if idx < self.tablesize - 1 { idx + 1 } else { 0 };
                    self.wavetable[idx] + (frac * (self.wavetable[next_idx] - self.wavetable[idx]))
                } * lvl_buf[sample_idx]; // apply oscillator level ...

                self.table_ptr += self.phase_inc;
                if self.table_ptr as usize >= self.tablesize {
                    self.table_ptr -= self.tablesize as f32;
                }
            }
        } else {
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
                } * self.lvl; // apply oscillator level ...

                self.table_ptr += self.phase_inc;
                if self.table_ptr as usize >= self.tablesize {
                    self.table_ptr -= self.tablesize as f32;
                }
            }
        }

        out_buf
    }
}
