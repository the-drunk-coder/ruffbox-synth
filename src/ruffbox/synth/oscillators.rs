use crate::ruffbox::synth::MonoSource;
use crate::ruffbox::synth::SynthParameter;

use std::f32::consts::PI;

/// A collection of oscillators, some of which are modeled
/// after scsynth, csound, etc ...

/**
 * A simple sine oscillator
 */
pub struct SineOsc<const BUFSIZE: usize> {
    lvl: f32,
    sin_time: f32,
    sin_delta_time: f32,
    pi_slice: f32,
    sample_count: u64,
}

impl<const BUFSIZE: usize> SineOsc<BUFSIZE> {
    pub fn new(freq: f32, lvl: f32, sr: f32) -> Self {
        SineOsc {
            lvl: lvl,
            sin_time: 0.0,
            sin_delta_time: 1.0 / sr,
            pi_slice: 2.0 * PI * freq,
            sample_count: 0,
        }
    }
}

impl<const BUFSIZE: usize> MonoSource<BUFSIZE> for SineOsc<BUFSIZE> {
    // some parameter limits might be nice ...
    fn set_parameter(&mut self, par: SynthParameter, value: f32) {
        match par {
            SynthParameter::PitchFrequency => self.pi_slice = 2.0 * PI * value,
            SynthParameter::Level => self.lvl = value,
            _ => (),
        };
    }

    fn finish(&mut self) {
        //self.state = SynthState::Finished;
    }

    fn is_finished(&self) -> bool {
        false
    }

    fn get_next_block(&mut self, start_sample: usize, _: &Vec<Vec<f32>>) -> [f32; BUFSIZE] {
        let mut out_buf: [f32; BUFSIZE] = [0.0; BUFSIZE];

        for i in start_sample..BUFSIZE {
            out_buf[i] =
                (self.pi_slice * self.sin_delta_time * self.sample_count as f32).sin() * self.lvl;
            self.sample_count += 1;
            self.sin_time += self.sin_delta_time;
        }

        out_buf
    }
}

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
    pub fn new(freq: f32, lvl: f32, sr: f32) -> Self {
        LFSaw {
            freq: freq,
            lvl: lvl,
            samplerate: sr,
            period_samples: (sr / freq).round() as usize,
            lvl_inc: (2.0 * lvl) / (sr / freq).round(),
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

    fn get_next_block(&mut self, start_sample: usize, _: &Vec<Vec<f32>>) -> [f32; BUFSIZE] {
        let mut out_buf: [f32; BUFSIZE] = [0.0; BUFSIZE];

        for i in start_sample..BUFSIZE {
            out_buf[i] = self.cur_lvl;
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

/**
 * A non-band-limited cubic sine approximation oscillator.
 */
pub struct LFCub<const BUFSIZE: usize> {
    lvl: f32,
    samplerate: f32,
    freq: f32,
    phase: f32,
}

impl<const BUFSIZE: usize> LFCub<BUFSIZE> {
    pub fn new(freq: f32, lvl: f32, sr: f32) -> Self {
        LFCub {
            //freq: freq,
            lvl: lvl,
            samplerate: sr,
            phase: 0.0,
            freq: freq,
        }
    }
}

impl<const BUFSIZE: usize> MonoSource<BUFSIZE> for LFCub<BUFSIZE> {
    // some parameter limits might be nice ...
    fn set_parameter(&mut self, par: SynthParameter, value: f32) {
        match par {
            SynthParameter::PitchFrequency => {
                self.freq = value * (1.0 / self.samplerate);
            }
            SynthParameter::Level => {
                self.lvl = value;
            }
            _ => (),
        };
    }

    fn finish(&mut self) {}

    fn is_finished(&self) -> bool {
        false
    }

    fn get_next_block(&mut self, start_sample: usize, _: &Vec<Vec<f32>>) -> [f32; BUFSIZE] {
        let mut out_buf: [f32; BUFSIZE] = [0.0; BUFSIZE];

        let mut z: f32;
        for i in start_sample..BUFSIZE {
            if self.phase < 1.0 {
                z = self.phase;
            } else if self.phase < 2.0 {
                z = 2.0 - self.phase;
            } else {
                self.phase -= 2.0;
                z = self.phase;
            }
            self.phase += self.freq;
            out_buf[i] = self.lvl * z * z * (6.0 - 4.0 * z) - 1.0;
        }

        out_buf
    }
}

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
    pub fn new(freq: f32, pw: f32, lvl: f32, sr: f32) -> Self {
        LFSquare {
            //freq: freq,
            lvl: lvl,
            samplerate: sr,
            pulsewidth: pw,
            period_samples: (sr / freq).round() as usize,
            period_count: 0,
            flank_point: ((sr / freq).round() * pw) as usize,
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
        };
    }

    fn finish(&mut self) {}

    fn is_finished(&self) -> bool {
        false
    }

    fn get_next_block(&mut self, start_sample: usize, _: &Vec<Vec<f32>>) -> [f32; BUFSIZE] {
        let mut out_buf: [f32; BUFSIZE] = [0.0; BUFSIZE];

        for i in start_sample..BUFSIZE {
            if self.period_count < self.flank_point {
                out_buf[i] = self.lvl;
            } else {
                out_buf[i] = -self.lvl;
            }

            self.period_count += 1;

            if self.period_count > self.period_samples {
                self.period_count = 0;
            }
        }

        out_buf
    }
}

// TEST TEST TEST
#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn sine_osc_test_at_block_start() {
        let mut osc = SineOsc::<128>::new(440.0, 1.0, 44100.0);

        let out_1 = osc.get_next_block(0, &Vec::new());
        let mut comp_1 = [0.0; 128];

        for i in 0..128 {
            comp_1[i] = (2.0 * PI * 440.0 * (i as f32 * (1.0 / 44100.0))).sin()
        }

        for i in 0..128 {
            //println!("{} {} {}; ", i, out_1[i], comp_1[i]);
            assert_approx_eq::assert_approx_eq!(out_1[i], comp_1[i], 0.00001);
        }
    }

    #[test]
    fn sine_osc_test_start_in_block() {
        let mut osc = SineOsc::<128>::new(440.0, 1.0, 44100.0);

        let start_time: f32 = 0.001;

        let sample_offset = (44100.0 * start_time).round() as usize;

        let out_1 = osc.get_next_block(sample_offset, &Vec::new());

        let mut comp_1 = [0.0; 128];

        for i in sample_offset..128 {
            comp_1[i] = (2.0 * PI * 440.0 * ((i - sample_offset) as f32 * (1.0 / 44100.0))).sin()
        }

        for i in 0..128 {
            //println!("{} {} {}; ", i, out_1[i], comp_1[i]);
            assert_approx_eq::assert_approx_eq!(out_1[i], comp_1[i], 0.00001);
        }
    }

    #[test]
    fn sine_osc_test_multiple_blocks() {
        let mut osc = SineOsc::<128>::new(440.0, 1.0, 44100.0);

        let out_1 = osc.get_next_block(0, &Vec::new());
        let out_2 = osc.get_next_block(0, &Vec::new());
        let out_3 = osc.get_next_block(0, &Vec::new());
        let out_4 = osc.get_next_block(0, &Vec::new());
        let out_5 = osc.get_next_block(0, &Vec::new());
        let out_6 = osc.get_next_block(0, &Vec::new());

        let mut comp_1 = [0.0; 128];
        let mut comp_2 = [0.0; 128];
        let mut comp_3 = [0.0; 128];
        let mut comp_4 = [0.0; 128];
        let mut comp_5 = [0.0; 128];
        let mut comp_6 = [0.0; 128];

        for i in 0..128 {
            comp_1[i] = (2.0 * PI * 440.0 * (i as f32 * (1.0 / 44100.0))).sin()
        }

        for i in 128..256 {
            comp_2[i - 128] = (2.0 * PI * 440.0 * (i as f32 * (1.0 / 44100.0))).sin()
        }

        for i in 256..384 {
            comp_3[i - 256] = (2.0 * PI * 440.0 * (i as f32 * (1.0 / 44100.0))).sin()
        }

        for i in 384..512 {
            comp_4[i - 384] = (2.0 * PI * 440.0 * (i as f32 * (1.0 / 44100.0))).sin()
        }

        for i in 512..640 {
            comp_5[i - 512] = (2.0 * PI * 440.0 * (i as f32 * (1.0 / 44100.0))).sin()
        }

        for i in 640..768 {
            comp_6[i - 640] = (2.0 * PI * 440.0 * (i as f32 * (1.0 / 44100.0))).sin()
        }

        for i in 0..128 {
            // this isn't very precise ???
            //println!("{} {} {}; ", i, out_1[i], comp_1[i]);
            assert_approx_eq::assert_approx_eq!(out_1[i], comp_1[i], 0.00001);
        }
        for i in 0..128 {
            // this isn't very precise ???
            //println!("{} {} {}; ", i, out_2[i], comp_2[i]);
            assert_approx_eq::assert_approx_eq!(out_2[i], comp_2[i], 0.00001);
        }
        for i in 0..128 {
            // this isn't very precise ???
            //println!("{} {} {}; ", i, out_3[i], comp_3[i]);
            assert_approx_eq::assert_approx_eq!(out_3[i], comp_3[i], 0.00001);
        }
        for i in 0..128 {
            // this isn't very precise ???
            //println!("{} {} {}; ", i, out_1[i], comp_1[i]);
            assert_approx_eq::assert_approx_eq!(out_4[i], comp_4[i], 0.00001);
        }
        for i in 0..128 {
            // this isn't very precise ???
            //println!("{} {} {}; ", i, out_2[i], comp_2[i]);
            assert_approx_eq::assert_approx_eq!(out_5[i], comp_5[i], 0.00001);
        }
        for i in 0..128 {
            // this isn't very precise ???
            //println!("{} {} {}; ", i, out_3[i], comp_3[i]);
            assert_approx_eq::assert_approx_eq!(out_6[i], comp_6[i], 0.0001);
        }
    }
}

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
    pub fn new(freq: f32, lvl: f32, sr: f32) -> Self {
	let period_samples = (sr / freq).round() as usize;		
	let segment_samples = period_samples / 4;
        LFTri {            
            lvl: lvl,
            samplerate: sr,
            segment_samples: segment_samples,
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
    fn set_parameter(&mut self, par: SynthParameter, value: f32) {
        match par {
            SynthParameter::PitchFrequency => {
		let period_samples = (self.samplerate / value).round() as usize;		
		// the segment-wise implementation is a bit strange but works for now ...
		self.segment_samples = period_samples / 4;		
		self.period_second_ascent_samples = period_samples;
		self.period_descent_samples = period_samples - self.segment_samples;
		self.period_first_ascent_samples = self.period_descent_samples - (2 * self.segment_samples);		
                self.lvl_inc_dec = self.lvl / self.segment_samples as f32;
		self.lvl_first_inc = self.lvl / self.period_first_ascent_samples as f32;		
            }
            SynthParameter::Level => {
                self.lvl = value;
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

    fn get_next_block(&mut self, start_sample: usize, _: &Vec<Vec<f32>>) -> [f32; BUFSIZE] {
        let mut out_buf: [f32; BUFSIZE] = [0.0; BUFSIZE];

        for i in start_sample..BUFSIZE {
            out_buf[i] = self.cur_lvl;
            self.period_count += 1;
            if self.period_count < self.period_first_ascent_samples {
		self.cur_lvl += self.lvl_first_inc;		
            } else if self.period_count > self.period_first_ascent_samples && self.period_count < self.period_descent_samples {
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
