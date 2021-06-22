// parent imports
use crate::ruffbox::synth::MonoSource;
use crate::ruffbox::synth::SynthParameter;
use crate::ruffbox::synth::SynthState;

/**
 * a very simple sample player ...
 */
pub struct Sampler<const BUFSIZE: usize> {
    index: usize,
    frac_index: f32,
    bufnum: usize,
    buflen: usize,    
    playback_rate: f32,
    frac_index_increment: f32,
    state: SynthState,
    level: f32,
    repeat: bool,
}

impl<const BUFSIZE: usize> Sampler<BUFSIZE> {
    pub fn with_bufnum_len(bufnum: usize, buflen: usize, repeat: bool) -> Sampler<BUFSIZE> {	
        Sampler {
            index: 1, // start with one to account for interpolation
            frac_index: 1.0,
            bufnum: bufnum,
	    buflen: buflen,
            playback_rate: 1.0,
            frac_index_increment: 1.0,
            state: SynthState::Fresh,
            level: 1.0,
            repeat: repeat,
        }
    }

    fn get_next_block_no_interp(&mut self, start_sample: usize, sample_buffers: &Vec<Vec<f32>>) -> [f32; BUFSIZE] {
        let mut out_buf: [f32; BUFSIZE] = [0.0; BUFSIZE];
	
        for i in start_sample..BUFSIZE {
            out_buf[i] = sample_buffers[self.bufnum][self.index] * self.level;

            if self.index < self.buflen {
                self.index = self.index + 1;
            } else {
                if self.repeat {
                    self.frac_index = 1.0;
                    self.index = 1;
                } else {
                    self.finish();
                }
            }
        }

        out_buf
    }

    fn get_next_block_interp(&mut self, start_sample: usize, sample_buffers: &Vec<Vec<f32>>) -> [f32; BUFSIZE] {
        let mut out_buf: [f32; BUFSIZE] = [0.0; BUFSIZE];
	
	for i in start_sample..BUFSIZE {
            // get sample:
            let idx = self.frac_index.floor();
            let frac = self.frac_index - idx;
            let idx_u = idx as usize;

            // 4-point, 3rd-order Hermite
            let y_m1 = sample_buffers[self.bufnum][idx_u - 1];
            let y_0 = sample_buffers[self.bufnum][idx_u];
            let y_1 = sample_buffers[self.bufnum][idx_u + 1];
            let y_2 = sample_buffers[self.bufnum][idx_u + 2];

            let c0 = y_0;
            let c1 = 0.5 * (y_1 - y_m1);
            let c2 = y_m1 - 2.5 * y_0 + 2.0 * y_1 - 0.5 * y_2;
            let c3 = 0.5 * (y_2 - y_m1) + 1.5 * (y_0 - y_1);

            out_buf[i] = (((c3 * frac + c2) * frac + c1) * frac + c0) * self.level;

            if ((self.frac_index + self.frac_index_increment) as usize) < self.buflen {
                self.frac_index = self.frac_index + self.frac_index_increment;
            } else {
                if self.repeat {
                    self.frac_index = 1.0;
                    self.index = 1;
                } else {
                    self.finish();
                }
            }
        }

        out_buf
    }
}

impl<const BUFSIZE: usize> MonoSource<BUFSIZE> for Sampler<BUFSIZE> {
    fn set_parameter(&mut self, par: SynthParameter, value: f32) {
        match par {
            SynthParameter::PlaybackStart => {
		let mut value_clamped = value;
		// clamp value
		if value > 1.0 {
		    value_clamped = value - ((value as usize) as f32);
		} else if value < 0.0 {
		    value_clamped = 1.0 + (value - ((value as i32) as f32));
		}
		
                let offset = ((self.buflen - 1) as f32 * value_clamped) as usize;
                self.index = offset + 1; // start counting at one, due to interpolation
		//println!("setting starting point to sample {}", self.index);
                self.frac_index = self.index as f32;
            }
            SynthParameter::PlaybackRate => {
                self.playback_rate = value;
                self.frac_index_increment = 1.0 * value;
            }
            SynthParameter::Level => {
                self.level = value;
            }
            _ => (),
        };
    }

    fn finish(&mut self) {
        self.state = SynthState::Finished;
    }

    fn is_finished(&self) -> bool {
        match self.state {
            SynthState::Finished => true,
            _ => false,
        }
    }

    fn get_next_block(&mut self, start_sample: usize, sample_buffers: &Vec<Vec<f32>>) -> [f32; BUFSIZE] {
        if self.playback_rate == 1.0 {
            self.get_next_block_no_interp(start_sample, sample_buffers)
        } else {
            self.get_next_block_interp(start_sample, sample_buffers)
        }
    }
}
