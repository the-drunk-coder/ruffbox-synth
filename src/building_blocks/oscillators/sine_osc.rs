use crate::building_blocks::{Modulator, MonoSource, SynthParameterLabel, SynthParameterValue};

use std::f32::consts::PI;

/**
 * A recursive sine oscillator
 * Based on equation (2) in this article:
 * https://www.dsprelated.com/freebooks/pasp/Digital_Sinusoid_Generators.html
 */
pub struct SineOsc<const BUFSIZE: usize> {
    lvl_buf: [f32; BUFSIZE],
    samplerate: f32,
    delta_t: f32,
    freq: f32,
    lvl: f32,
    x1_last: f32, // delay line
    x2_last: f32, // delay line 
    mcf_buf: [f32; BUFSIZE], // the "magic circle" factors
    freq_mod: Option<Modulator<BUFSIZE>>, // currently allows modulating frequency ..    
    lvl_mod: Option<Modulator<BUFSIZE>>, // and level
}

impl<const BUFSIZE: usize> SineOsc<BUFSIZE> {
    pub fn new(freq: f32, lvl: f32, sr: f32) -> Self {
        SineOsc {
            lvl,
            lvl_buf: [lvl; BUFSIZE],
            delta_t: 1.0 / sr,
            samplerate: sr,
            x1_last: 0.0,
            x2_last: 1.0,
            mcf_buf: [2.0 * (PI * freq * 1.0 / sr).sin(); BUFSIZE],
            freq,
            freq_mod: None,
            lvl_mod: None,
        }
    }
}

impl<const BUFSIZE: usize> MonoSource<BUFSIZE> for SineOsc<BUFSIZE> {
    // some parameter limits might be nice ...
    fn set_parameter(&mut self, par: SynthParameterLabel, value: &SynthParameterValue) {
        match par {
            SynthParameterLabel::PitchFrequency => {
                match value {
                    SynthParameterValue::ScalarF32(f) => {
                        self.freq = *f;
                        self.mcf_buf = [2.0 * (PI * f * 1.0 / self.samplerate).sin(); BUFSIZE];
                    }
                    SynthParameterValue::Lfo(init, freq, range, op) => {
			self.freq = *init;
                        self.freq_mod = Some(Modulator::lfo(*op, *freq, *range, self.samplerate))
                    }
                    _ => { /* nothing to do, don't know how to handle this ... */ }
                }
            }
            SynthParameterLabel::Level => {
                match value {
                    SynthParameterValue::ScalarF32(l) => {
                        self.lvl = l.clamp(-1.2, 1.2);
                        self.lvl_buf = [self.lvl; BUFSIZE];
                    }
                    SynthParameterValue::Lfo(init, freq, range, op) => {
			self.lvl = init.clamp(-1.2, 1.2);
                        // clamp to reasonable value ...
                        self.lvl_mod = Some(Modulator::lfo(
                            *op,
                            *freq,
                            range.clamp(-1.2, 1.2),
                            self.samplerate,
                        ))
                    }
                    _ => { /* nothing to do, don't know how to handle this ... */ }
                }
            }
            _ => (),
        };
    }

    fn finish(&mut self) {
        //self.state = SynthState::Finished;
    }

    fn is_finished(&self) -> bool {
        false
    }

    fn get_next_block(&mut self, start_sample: usize, in_buffers: &[Vec<f32>]) -> [f32; BUFSIZE] {
        let mut out_buf: [f32; BUFSIZE] = [0.0; BUFSIZE];
	
        if self.freq_mod.is_some() {
	    // re-calculate magic circle factors if we have a
	    // modulated frequency
            self.mcf_buf = self
                .freq_mod
                .as_mut()
                .unwrap()
                .process(self.freq, start_sample, in_buffers)
                .map(|f| 2.0 * (PI * f * self.delta_t).sin());
        }

        if self.lvl_mod.is_some() {
	    // recalculate levels if we have modulated levels
            self.lvl_buf =
                self.lvl_mod
                    .as_mut()
                    .unwrap()
                    .process(self.lvl, start_sample, in_buffers);
        }

        for (idx, current_sample) in out_buf
            .iter_mut()
            .enumerate()
            .take(BUFSIZE)
            .skip(start_sample)
        {
            let x1 = self.x1_last + (self.mcf_buf[idx] * self.x2_last);
            let x2 = -self.mcf_buf[idx] * x1 + self.x2_last;
            *current_sample = x2 * self.lvl_buf[idx];
            self.x1_last = x1;
            self.x2_last = x2;
        }

        out_buf
    }
}
