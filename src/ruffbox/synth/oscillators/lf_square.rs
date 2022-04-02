use crate::ruffbox::synth::MonoSource;
use crate::ruffbox::synth::SynthParameter;

/**
 * A non-band-limited square-wave oscillator.
 */
pub struct LFSquare<const BUFSIZE: usize> {
    //freq: f32,
    lvl: f32,
    samplerate: f32,
    pulsewidth: f32,
    period_samples: usize,
    period_count: usize,
    flank_point: usize,
}

impl<const BUFSIZE: usize> LFSquare<BUFSIZE> {
    pub fn new(freq: f32, pulsewidth: f32, lvl: f32, samplerate: f32) -> Self {
        LFSquare {
            //freq: freq,
            lvl,
            samplerate,
            pulsewidth,
            period_samples: (samplerate / freq).round() as usize,
            period_count: 0,
            flank_point: ((samplerate / freq).round() * pulsewidth) as usize,
        }
    }
}

impl<const BUFSIZE: usize> MonoSource<BUFSIZE> for LFSquare<BUFSIZE> {
    // some parameter limits might be nice ...
    fn set_parameter(&mut self, par: SynthParameter, value: f32) {
        match par {
            SynthParameter::PitchFrequency => {
                //self.freq = value;
                self.period_samples = (self.samplerate / value).round() as usize;
                self.flank_point = (self.period_samples as f32 * self.pulsewidth).round() as usize;
            }
            SynthParameter::Pulsewidth => {
                self.pulsewidth = value;
                self.flank_point = (self.period_samples as f32 * value).round() as usize;
            }
            SynthParameter::Level => {
                self.lvl = value;
            }
            _ => (),
        }
    }

    fn finish(&mut self) {}

    fn is_finished(&self) -> bool {
        false
    }

    fn get_next_block(&mut self, start_sample: usize, _: &[Vec<f32>]) -> [f32; BUFSIZE] {
        let mut out_buf: [f32; BUFSIZE] = [0.0; BUFSIZE];

        for current_sample in out_buf.iter_mut().take(BUFSIZE).skip(start_sample) {
            if self.period_count < self.flank_point {
                *current_sample = self.lvl;
            } else {
                *current_sample = -self.lvl;
            }

            self.period_count += 1;

            if self.period_count > self.period_samples {
                self.period_count = 0;
            }
        }

        out_buf
    }
}