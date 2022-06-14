use crate::building_blocks::{
    Modulator, MonoSource, SynthParameterLabel, SynthParameterValue, SynthState, ValueOrModulator,
};

/**
 * A simple, raw wavetable oscillator
 */
#[derive(Clone)]
pub struct Wavetable<const BUFSIZE: usize> {
    // user parameters
    amp: f32,
    freq: f32,
    wavetable: [f32; 2048], // max len

    // internal parameters
    tablesize: usize,
    phase_inc: f32,
    table_ptr: f32,
    state: SynthState,
    sample_period: f32,
    //samplerate: f32,

    // modulator slots
    freq_mod: Option<Modulator<BUFSIZE>>,
    amp_mod: Option<Modulator<BUFSIZE>>,
}

impl<const BUFSIZE: usize> Wavetable<BUFSIZE> {
    pub fn new(sr: f32) -> Wavetable<BUFSIZE> {
        Wavetable {
            freq: 46.875,
            amp: 1.0,
            wavetable: [0.5; 2048],
            tablesize: 2048,
            phase_inc: 1.0,
            table_ptr: 0.0,
            state: SynthState::Fresh,
            sample_period: 1.0 / sr,
            //samplerate: sr,
            freq_mod: None,
            amp_mod: None,
        }
    }
}

impl<const BUFSIZE: usize> MonoSource<BUFSIZE> for Wavetable<BUFSIZE> {
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
            SynthParameterLabel::PitchFrequency => {
                self.freq = init;
                self.phase_inc = self.tablesize as f32 * self.freq * self.sample_period;
                self.freq_mod = Some(modulator);
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
            SynthParameterLabel::PitchFrequency => {
                if let SynthParameterValue::ScalarF32(value) = val {
                    self.freq = *value;
                    self.phase_inc = self.tablesize as f32 * self.freq * self.sample_period;
                }
            }
            SynthParameterLabel::Wavetable => {
                if let SynthParameterValue::VecF32(tab) = val {
                    self.tablesize = std::cmp::min(tab.len(), 2048);
                    self.wavetable[..self.tablesize].copy_from_slice(&tab[..self.tablesize]);
                    self.phase_inc = self.tablesize as f32 * self.freq * self.sample_period;
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

    fn get_next_block(&mut self, start_sample: usize, in_buffers: &[Vec<f32>]) -> [f32; BUFSIZE] {
        let mut out_buf: [f32; BUFSIZE] = [0.0; BUFSIZE];

        if self.freq_mod.is_some() || self.amp_mod.is_some() {
            let amp_buf = if let Some(m) = self.amp_mod.as_mut() {
                m.process(self.amp, start_sample, in_buffers)
            } else {
                [self.amp; BUFSIZE]
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
                } * amp_buf[sample_idx]; // apply oscillator level ...

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
                } * self.amp; // apply oscillator level ...

                self.table_ptr += self.phase_inc;
                if self.table_ptr as usize >= self.tablesize {
                    self.table_ptr -= self.tablesize as f32;
                }
            }
        }

        out_buf
    }
}
