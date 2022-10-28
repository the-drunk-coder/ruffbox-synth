use crate::building_blocks::{
    Modulator, MonoSource, SynthParameterLabel, SynthParameterValue, ValueOrModulator,
};

use std::f32::consts::PI;
//use std::f32::consts::FRAC_PI_2;

/**
 * A recursive sine oscillator
 * Based on equation (2) in this article:
 * https://www.dsprelated.com/freebooks/pasp/Digital_Sinusoid_Generators.html
 */
#[derive(Clone)]
pub struct SineOsc<const BUFSIZE: usize> {
    // user parameters
    freq: f32,
    amp: f32,

    // internal parameters
    amp_buf: [f32; BUFSIZE],
    samplerate: f32,
    delta_t: f32,
    x1_last: f32,            // delay line
    x2_last: f32,            // delay line
    mcf_buf: [f32; BUFSIZE], // the "magic circle" factors

    // modulator slots
    freq_mod: Option<Modulator<BUFSIZE>>, // currently allows modulating frequency ..
    amp_mod: Option<Modulator<BUFSIZE>>,  // and level
}

impl<const BUFSIZE: usize> SineOsc<BUFSIZE> {
    pub fn new(freq: f32, amp: f32, sr: f32) -> Self {
        SineOsc {
            freq,
            amp,
            amp_buf: [amp; BUFSIZE],
            delta_t: 1.0 / sr,
            samplerate: sr,
            x1_last: ((-2.0 * PI * freq) / sr).cos(),
            x2_last: ((-2.0 * PI * freq) / sr).sin(),
            mcf_buf: [-2.0 * (PI * (freq / sr)).sin(); BUFSIZE],
            freq_mod: None,
            amp_mod: None,
        }
    }
}

impl<const BUFSIZE: usize> MonoSource<BUFSIZE> for SineOsc<BUFSIZE> {
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
                self.freq_mod = Some(modulator);
            }
            SynthParameterLabel::OscillatorAmplitude => {
                self.amp = init;
                self.amp_mod = Some(modulator);
            }
            _ => {}
        }
    }
    // some parameter limits might be nice ...
    fn set_parameter(&mut self, par: SynthParameterLabel, value: &SynthParameterValue) {
        match par {
            // set phase to a value relative to one full wave period,
            // i.e 0 = 0.0, 0.25 = 1.0, etc
            SynthParameterLabel::OscillatorPhaseRelative => {
                if let SynthParameterValue::ScalarF32(p) = value {
                    self.x1_last = ((-2.0 * PI * self.freq / self.samplerate) + (p * PI)).cos();
                    self.x2_last = ((-2.0 * PI * self.freq / self.samplerate) + (p * PI)).sin();
                }
            }
            // set the phase to an absolute value.
            // only makes sense in conjunction with the amplitude
            SynthParameterLabel::OscillatorPhaseEffective => {
                if let SynthParameterValue::ScalarF32(p) = value {
                    self.x1_last =
                        ((-2.0 * PI * self.freq / self.samplerate) + (p / self.amp)).cos();
                    self.x2_last =
                        ((-2.0 * PI * self.freq / self.samplerate) + (p / self.amp)).sin();
                }
            }
            SynthParameterLabel::PitchFrequency => {
                if let SynthParameterValue::ScalarF32(f) = value {
                    self.freq = *f;
                    self.x1_last = ((-2.0 * PI * self.freq) / self.samplerate).cos();
                    self.x2_last = ((-2.0 * PI * self.freq) / self.samplerate).sin();
                    self.mcf_buf = [-2.0 * (PI * (self.freq / self.samplerate)).sin(); BUFSIZE];
                }
            }
            SynthParameterLabel::OscillatorAmplitude => {
                if let SynthParameterValue::ScalarF32(l) = value {
                    self.amp = *l;
                    self.amp_buf = [self.amp; BUFSIZE];
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
                .map(|f| -2.0 * (PI * f * self.delta_t).sin());
        }

        if self.amp_mod.is_some() {
            // recalculate levels if we have modulated levels
            self.amp_buf =
                self.amp_mod
                    .as_mut()
                    .unwrap()
                    .process(self.amp, start_sample, in_buffers);
        }
        //println!("{:?}\n\n", self.mcf_buf);
        for (idx, current_sample) in out_buf
            .iter_mut()
            .enumerate()
            .take(BUFSIZE)
            .skip(start_sample)
        {
            let x1 = self.x1_last + (self.mcf_buf[idx] * self.x2_last);
            let x2 = -self.mcf_buf[idx] * x1 + self.x2_last;

            *current_sample = x2 * self.amp_buf[idx];
            //println!("x1 {} x2 {} cur {}", x1, x2, current_sample);
            //debug_plotter::plot!(x1, x2 where caption = "IntPlot");
            self.x1_last = x1;
            self.x2_last = x2;
        }

        out_buf
    }
}
