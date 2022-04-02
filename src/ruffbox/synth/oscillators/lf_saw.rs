use crate::ruffbox::synth::MonoSource;
use crate::ruffbox::synth::SynthParameter;

/**
 * A non-band-limited sawtooth oscillator.
 */
pub struct LFSaw<const BUFSIZE: usize> {
    freq: f32,
    lvl: f32,
    samplerate: f32,
    period_samples: usize,
    lvl_inc: f32,
    cur_lvl: f32,
    period_count: usize,
}

impl<const BUFSIZE: usize> LFSaw<BUFSIZE> {
    pub fn new(freq: f32, lvl: f32, samplerate: f32) -> Self {
        LFSaw {
            freq,
            lvl,
            samplerate,
            period_samples: (samplerate / freq).round() as usize,
            lvl_inc: (2.0 * lvl) / (samplerate / freq).round(),
            cur_lvl: -1.0 * lvl,
            period_count: 0,
        }
    }
}

impl<const BUFSIZE: usize> MonoSource<BUFSIZE> for LFSaw<BUFSIZE> {
    // some parameter limits might be nice ...
    fn set_parameter(&mut self, par: SynthParameter, value: f32) {
        match par {
            SynthParameter::PitchFrequency => {
                self.freq = value;
                self.period_samples = (self.samplerate / value).round() as usize;
                self.lvl_inc = (2.0 * self.lvl) / (self.samplerate / value).round();
            }
            SynthParameter::Level => {
                self.lvl = value;
                self.lvl_inc = (2.0 * self.lvl) / (self.samplerate / self.freq).round();
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
            if self.period_count > self.period_samples {
                self.period_count = 0;
                self.cur_lvl = -1.0 * self.lvl;
            } else {
                self.cur_lvl += self.lvl_inc;
            }
        }

        out_buf
    }
}