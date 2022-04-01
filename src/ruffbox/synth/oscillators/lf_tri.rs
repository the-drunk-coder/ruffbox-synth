use crate::ruffbox::synth::MonoSource;
use crate::ruffbox::synth::SynthParameter;

/**
 * A non-band-limited triangle oscillator.
 */
pub struct LFTri<const BUFSIZE: usize> {
    lvl: f32,
    samplerate: f32,
    // ascent, descent, ascent ...
    segment_samples: usize,
    period_first_ascent_samples: usize,
    period_second_ascent_samples: usize,
    period_descent_samples: usize,
    lvl_first_inc: f32,
    lvl_inc_dec: f32,
    cur_lvl: f32,
    period_count: usize,
}

impl<const BUFSIZE: usize> LFTri<BUFSIZE> {
    pub fn new(freq: f32, lvl: f32, samplerate: f32) -> Self {
        let period_samples = (samplerate / freq).round() as usize;
        let segment_samples = period_samples / 4;
        LFTri {
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
        }
    }
}

impl<const BUFSIZE: usize> MonoSource<BUFSIZE> for LFTri<BUFSIZE> {
    // some parameter limits might be nice ...
    fn set_parameter(&mut self, par: SynthParameter) {
        match par {
            SynthParameter::PitchFrequency(f) => {
                let period_samples = (self.samplerate / f).round() as usize;
                // the segment-wise implementation is a bit strange but works for now ...
                self.segment_samples = period_samples / 4;
                self.period_second_ascent_samples = period_samples;
                self.period_descent_samples = period_samples - self.segment_samples;
                self.period_first_ascent_samples =
                    self.period_descent_samples - (2 * self.segment_samples);
                self.lvl_inc_dec = self.lvl / self.segment_samples as f32;
                self.lvl_first_inc = self.lvl / self.period_first_ascent_samples as f32;
            }
            SynthParameter::Level(l) => {
                self.lvl = l;
                self.lvl_inc_dec = self.lvl / self.segment_samples as f32;
                self.lvl_first_inc = self.lvl / self.period_first_ascent_samples as f32;
            }
            _ => (),
        };
    }

    fn finish(&mut self) {}

    fn is_finished(&self) -> bool {
        false
    }

    fn get_next_block(&mut self, start_sample: usize, _: &[Vec<f32>]) -> [f32; BUFSIZE] {
        let mut out_buf: [f32; BUFSIZE] = [0.0; BUFSIZE];

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

        out_buf
    }
}
