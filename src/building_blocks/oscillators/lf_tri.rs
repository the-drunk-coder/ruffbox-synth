use crate::building_blocks::{Modulator, MonoSource, SynthParameterLabel, SynthParameterValue};

/**
 * A non-band-limited triangle oscillator.
 */
pub struct LFTri<const BUFSIZE: usize> {
    // user parameters
    freq: f32,
    lvl: f32,

    // internal parameters
    samplerate: f32,
    segment_samples: usize,
    // Ascent, Descent, Ascent ...
    period_first_ascent_samples: usize,
    period_second_ascent_samples: usize,
    period_descent_samples: usize,
    lvl_first_inc: f32,
    lvl_inc_dec: f32,
    cur_lvl: f32,
    period_count: usize,

    // modulator slots
    freq_mod: Option<Modulator<BUFSIZE>>, // allows modulating frequency ..
    lvl_mod: Option<Modulator<BUFSIZE>>,  // and level
}

impl<const BUFSIZE: usize> LFTri<BUFSIZE> {
    pub fn new(freq: f32, lvl: f32, samplerate: f32) -> Self {
        let period_samples = (samplerate / freq).round() as usize;
        let segment_samples = period_samples / 4;
        LFTri {
            freq,
            lvl,
            samplerate,
            segment_samples,
            period_first_ascent_samples: period_samples - (3 * segment_samples),
            period_second_ascent_samples: period_samples,
            period_descent_samples: period_samples - segment_samples,
            lvl_first_inc: lvl / (period_samples - (3 * segment_samples)) as f32,
            lvl_inc_dec: lvl / segment_samples as f32,
            cur_lvl: 0.0,
            period_count: 0,
            freq_mod: None,
            lvl_mod: None,
        }
    }

    fn update_internals(&mut self, freq: f32, lvl: f32) {
        let period_samples = (self.samplerate / freq).round() as usize;
        // the segment-wise implementation is a bit strange but works for now ...
        self.segment_samples = period_samples / 4;
        self.period_second_ascent_samples = period_samples;
        self.period_descent_samples = period_samples - self.segment_samples;
        self.period_first_ascent_samples = self.period_descent_samples - (2 * self.segment_samples);
        self.lvl_inc_dec = lvl / self.segment_samples as f32;
        self.lvl_first_inc = lvl / self.period_first_ascent_samples as f32;
    }
}

impl<const BUFSIZE: usize> MonoSource<BUFSIZE> for LFTri<BUFSIZE> {
    // some parameter limits might be nice ...
    fn set_parameter(&mut self, par: SynthParameterLabel, value: &SynthParameterValue) {
        match par {
            SynthParameterLabel::PitchFrequency => match value {
                SynthParameterValue::ScalarF32(f) => {
                    self.freq = *f;
                    self.update_internals(self.freq, self.lvl);
                }
                SynthParameterValue::Lfo(init, freq, amp, add, op) => {
                    self.freq = *init;
                    self.freq_mod = Some(Modulator::lfo(*op, *freq, *amp, *add, self.samplerate))
                }
                SynthParameterValue::LFSaw(init, freq, amp, add, op) => {
                    self.freq = *init;
                    self.freq_mod = Some(Modulator::lfsaw(*op, *freq, *amp, *add, self.samplerate))
                }
                SynthParameterValue::LFTri(init, freq, amp, add, op) => {
                    self.freq = *init;
                    self.freq_mod = Some(Modulator::lftri(*op, *freq, *amp, *add, self.samplerate))
                }
                SynthParameterValue::LFSquare(init, freq, pw, amp, add, op) => {
                    self.freq = *init;
                    self.freq_mod = Some(Modulator::lfsquare(
                        *op,
                        *freq,
                        *pw,
                        *amp,
                        *add,
                        self.samplerate,
                    ))
                }
                _ => {}
            },
            SynthParameterLabel::OscillatorLevel => match value {
                SynthParameterValue::ScalarF32(l) => {
                    self.lvl = *l;
                    self.update_internals(self.freq, self.lvl);
                }
                SynthParameterValue::Lfo(init, freq, amp, add, op) => {
                    self.lvl = *init;
                    self.lvl_mod = Some(Modulator::lfo(*op, *freq, *amp, *add, self.samplerate))
                }
                SynthParameterValue::LFTri(init, freq, amp, add, op) => {
                    self.lvl = *init;
                    self.lvl_mod = Some(Modulator::lftri(*op, *freq, *amp, *add, self.samplerate))
                }
                SynthParameterValue::LFSaw(init, freq, amp, add, op) => {
                    self.lvl = *init;
                    self.lvl_mod = Some(Modulator::lfsaw(*op, *freq, *amp, *add, self.samplerate))
                }
                SynthParameterValue::LFSquare(init, freq, pw, amp, add, op) => {
                    self.lvl = *init;
                    self.lvl_mod = Some(Modulator::lfsquare(
                        *op,
                        *freq,
                        *pw,
                        *amp,
                        *add,
                        self.samplerate,
                    ))
                }
                _ => {}
            },
            _ => (),
        };
    }

    fn finish(&mut self) {}

    fn is_finished(&self) -> bool {
        false
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

            for (idx, current_sample) in out_buf
                .iter_mut()
                .enumerate()
                .take(BUFSIZE)
                .skip(start_sample)
            {
                self.update_internals(freq_buf[idx], lvl_buf[idx]);

                *current_sample = self.cur_lvl;

                self.period_count += 1;
                if self.period_count < self.period_first_ascent_samples {
                    self.cur_lvl += self.lvl_first_inc;
                } else if self.period_count > self.period_first_ascent_samples
                    && self.period_count < self.period_descent_samples
                {
                    self.cur_lvl -= self.lvl_inc_dec;
                } else if self.period_count < self.period_second_ascent_samples {
                    self.cur_lvl += self.lvl_inc_dec;
                } else {
                    self.period_count = 0;
                    self.cur_lvl = 0.0;
                }
            }
        } else {
            for current_sample in out_buf.iter_mut().take(BUFSIZE).skip(start_sample) {
                *current_sample = self.cur_lvl;

                self.period_count += 1;
                if self.period_count < self.period_first_ascent_samples {
                    self.cur_lvl += self.lvl_first_inc;
                } else if self.period_count > self.period_first_ascent_samples
                    && self.period_count < self.period_descent_samples
                {
                    self.cur_lvl -= self.lvl_inc_dec;
                } else if self.period_count < self.period_second_ascent_samples {
                    self.cur_lvl += self.lvl_inc_dec;
                } else {
                    self.period_count = 0;
                    self.cur_lvl = 0.0;
                }
            }
        }

        out_buf
    }
}
