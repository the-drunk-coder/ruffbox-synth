use crate::building_blocks::{Modulator, MonoSource, SynthParameterLabel, SynthParameterValue};

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
    freq_mod: Option<Modulator<BUFSIZE>>, // currently allows modulating frequency ..
    lvl_mod: Option<Modulator<BUFSIZE>>,  // and level
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
	    freq_mod: None,
            lvl_mod: None,
        }
    }
}

impl<const BUFSIZE: usize> MonoSource<BUFSIZE> for LFSaw<BUFSIZE> {
    // some parameter limits might be nice ...
    fn set_parameter(&mut self, par: SynthParameterLabel, value: &SynthParameterValue) {
        match par {
            SynthParameterLabel::PitchFrequency => {
		match value {
		    SynthParameterValue::ScalarF32(f) => {
			self.freq = *f;
			self.period_samples = (self.samplerate / f).round() as usize;
			self.lvl_inc = (2.0 * self.lvl) / (self.samplerate / f).round();
                    }
		    SynthParameterValue::Lfo(init, freq, range, op) => {
                        self.freq = *init;
                        self.freq_mod = Some(Modulator::lfo(*op, *freq, *range, self.samplerate))
                    }
		    _ => {},
		}                                    
            }
            SynthParameterLabel::Level => {
		match value {
		    SynthParameterValue::ScalarF32(l) => {
			self.lvl = *l;
			self.lvl_inc = (2.0 * self.lvl) / (self.samplerate / self.freq).round();
                    }
		    SynthParameterValue::Lfo(init, freq, range, op) => {
                        self.lvl = *init;
                        self.lvl_mod = Some(Modulator::lfo(*op, *freq, *range, self.samplerate))
                    }
		    _ => {},
		}
                
            }
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

	    for (idx, current_sample) in out_buf.iter_mut().enumerate().take(BUFSIZE).skip(start_sample) {
		*current_sample = self.cur_lvl;
		self.period_samples = (self.samplerate / freq_buf[idx]).round() as usize;
		self.lvl_inc = (2.0 * lvl_buf[idx]) / self.period_samples as f32;
		self.period_count += 1;
		if self.period_count > self.period_samples {
                    self.period_count = 0;
                    self.cur_lvl = -1.0 * lvl_buf[idx];
		} else {
                    self.cur_lvl += self.lvl_inc;
		}
            }
	    
	} else {
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
	}
	    

        

        out_buf
    }
}
