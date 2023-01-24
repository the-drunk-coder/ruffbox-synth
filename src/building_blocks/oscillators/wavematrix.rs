use crate::building_blocks::{
    Modulator, MonoSource, SynthParameterLabel, SynthParameterValue, SynthState,
};

/**
 * A 2D wavetable oscillator
 */
#[derive(Clone)]
pub struct Wavematrix<const BUFSIZE: usize> {
    // user parameters
    amp: f32,
    freq: f32,
    table_idx: f32,
    wavematrix: Vec<[f32; 2048]>, // max len

    // internal parameters
    tablesize: usize,
    matrixsize: usize, // terminology isn't super-precise here ...
    phase_inc_smp: f32,
    phase_inc_tab: f32,
    sample_ptr: f32, // for the inner tables
    state: SynthState,
    sample_period: f32,
    //samplerate: f32,

    // modulator slots
    freq_mod: Option<Modulator<BUFSIZE>>,
    amp_mod: Option<Modulator<BUFSIZE>>,
    table_idx_mod: Option<Modulator<BUFSIZE>>,
}

impl<const BUFSIZE: usize> Wavematrix<BUFSIZE> {
    pub fn new(sr: f32) -> Wavematrix<BUFSIZE> {
        Wavematrix {
            freq: 46.875,
            amp: 1.0,
            table_idx: 0.0,
            wavematrix: vec![[0.5; 2048]],
            tablesize: 2048,
            matrixsize: 1,
            phase_inc_smp: 1.0,
            phase_inc_tab: 0.0,
            sample_ptr: 0.0,
            state: SynthState::Fresh,
            sample_period: 1.0 / sr,
            //samplerate: sr,
            freq_mod: None,
            amp_mod: None,
            table_idx_mod: None,
        }
    }
}

impl<const BUFSIZE: usize> MonoSource<BUFSIZE> for Wavematrix<BUFSIZE> {
    fn reset(&mut self) {}

    fn set_modulator(
        &mut self,
        par: SynthParameterLabel,
        init: f32,
        modulator: Modulator<BUFSIZE>,
    ) {
        match par {
            SynthParameterLabel::PitchFrequency => {
                self.freq = init;
                self.phase_inc_smp = self.tablesize as f32 * self.freq * self.sample_period;
                self.freq_mod = Some(modulator);
            }
            SynthParameterLabel::OscillatorAmplitude => {
                self.amp = init;
                self.amp_mod = Some(modulator);
            }
            SynthParameterLabel::WavematrixTableIndex => {
                self.table_idx = init;
                self.table_idx_mod = Some(modulator);
            }
            _ => {}
        }
    }

    fn set_parameter(&mut self, par: SynthParameterLabel, val: &SynthParameterValue) {
        match par {
            SynthParameterLabel::PitchFrequency => {
                if let SynthParameterValue::ScalarF32(value) = val {
                    self.freq = *value;
                    self.phase_inc_smp = self.tablesize as f32 * self.freq * self.sample_period;
                }
            }
            SynthParameterLabel::Wavetable => {
                if let SynthParameterValue::VecF32(tab) = val {
                    self.tablesize = std::cmp::min(tab.len(), 2048);
                    self.wavematrix[0][..self.tablesize].copy_from_slice(&tab[..self.tablesize]);
                    self.phase_inc_smp = self.tablesize as f32 * self.freq * self.sample_period;
                }
            }
            SynthParameterLabel::Wavematrix => {
                if let SynthParameterValue::MatrixF32((outer, inner), mat) = val {
                    self.tablesize = std::cmp::min(*inner, 2048);
                    self.matrixsize = std::cmp::min(*outer, mat.len());
                    self.wavematrix = Vec::new();
                    for (i, row) in mat.iter().enumerate().take(self.matrixsize) {
                        self.wavematrix.push([0.0; 2048]);
                        self.wavematrix[i][..self.tablesize]
                            .copy_from_slice(&row[..self.tablesize]);
                    }

                    self.phase_inc_smp = self.tablesize as f32 * self.freq * self.sample_period;
                }
            }
            SynthParameterLabel::WavematrixTableIndex => {
                if let SynthParameterValue::ScalarF32(value) = val {
                    self.table_idx = *value;
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
        in_buffers: &[SampleBuffer],
    ) -> [f32; BUFSIZE] {
        let mut out_buf: [f32; BUFSIZE] = [0.0; BUFSIZE];

        if self.freq_mod.is_some() || self.amp_mod.is_some() || self.table_idx_mod.is_some() {
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

            let table_idx_buf = if let Some(m) = self.table_idx_mod.as_mut() {
                m.process(self.table_idx, start_sample, in_buffers)
            } else {
                [self.table_idx; BUFSIZE]
            };

            for (sample_idx, current_sample) in out_buf
                .iter_mut()
                .enumerate()
                .take(BUFSIZE)
                .skip(start_sample)
            {
                self.phase_inc_smp =
                    self.tablesize as f32 * freq_buf[sample_idx] * self.sample_period;
                self.phase_inc_tab =
                    self.matrixsize as f32 * table_idx_buf[sample_idx] * self.sample_period;

                // get sample:
                // sample index
                let smp_idx = self.sample_ptr as usize;
                let smp_frac = self.sample_ptr - (smp_idx as f32);

                // table index
                let tab_idx = table_idx_buf[sample_idx] as usize;
                let tab_frac = table_idx_buf[sample_idx] - (tab_idx as f32);

                // use simple linear interpolation for now ...
                *current_sample = if smp_frac == 0.0 && tab_frac == 0.0 {
                    self.wavematrix[tab_idx][smp_idx]
                } else {
                    let next_smp_idx = if smp_idx < self.tablesize - 1 {
                        smp_idx + 1
                    } else {
                        0
                    };
                    let next_tab_idx = if tab_idx < self.matrixsize - 1 {
                        tab_idx + 1
                    } else {
                        0
                    };

                    let smp1 = self.wavematrix[tab_idx][smp_idx]
                        + (smp_frac
                            * (self.wavematrix[tab_idx][next_smp_idx]
                                - self.wavematrix[tab_idx][smp_idx]));
                    let smp2 = self.wavematrix[next_tab_idx][smp_idx]
                        + (smp_frac
                            * (self.wavematrix[next_tab_idx][next_smp_idx]
                                - self.wavematrix[next_tab_idx][smp_idx]));
                    smp1 + (tab_frac * (smp2 - smp1))
                } * amp_buf[sample_idx]; // apply oscillator level ...

                self.sample_ptr += self.phase_inc_smp;
                if self.sample_ptr as usize >= self.tablesize {
                    self.sample_ptr -= self.tablesize as f32;
                }
            }
        } else {
            for current_sample in out_buf.iter_mut().take(BUFSIZE).skip(start_sample) {
                // get sample:
                // sample index
                let smp_idx = self.sample_ptr as usize;
                let smp_frac = self.sample_ptr - (smp_idx as f32);

                // table index
                let tab_idx = self.table_idx as usize;
                let tab_frac = self.table_idx - (tab_idx as f32);

                // use simple linear interpolation for now ...
                *current_sample = if smp_frac == 0.0 && tab_frac == 0.0 {
                    self.wavematrix[tab_idx][smp_idx]
                } else {
                    let next_smp_idx = if smp_idx < self.tablesize - 1 {
                        smp_idx + 1
                    } else {
                        0
                    };
                    let next_tab_idx = if tab_idx < self.matrixsize - 1 {
                        tab_idx + 1
                    } else {
                        0
                    };

                    let smp1 = self.wavematrix[tab_idx][smp_idx]
                        + (smp_frac
                            * (self.wavematrix[tab_idx][next_smp_idx]
                                - self.wavematrix[tab_idx][smp_idx]));
                    let smp2 = self.wavematrix[next_tab_idx][smp_idx]
                        + (smp_frac
                            * (self.wavematrix[next_tab_idx][next_smp_idx]
                                - self.wavematrix[next_tab_idx][smp_idx]));
                    smp1 + (tab_frac * (smp2 - smp1))
                } * self.amp; // apply oscillator level ...

                self.sample_ptr += self.phase_inc_smp;
                if self.sample_ptr as usize >= self.tablesize {
                    self.sample_ptr -= self.tablesize as f32;
                }
            }
        }

        out_buf
    }
}
