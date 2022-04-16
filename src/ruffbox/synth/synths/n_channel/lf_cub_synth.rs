use crate::ruffbox::synth::envelopes::*;
use crate::ruffbox::synth::filters::*;
use crate::ruffbox::synth::oscillators::*;
use crate::ruffbox::synth::routing::PanChan;
use crate::ruffbox::synth::*;
use crate::ruffbox::synth::{Modulator, Synth};
use crate::ruffbox::synth::{SynthParameterLabel, SynthParameterValue};

/// a cubic sine approximation synth with envelope etc.
pub struct LFCubSynth<const BUFSIZE: usize, const NCHAN: usize> {
    modulators: Vec<Modulator<BUFSIZE>>,
    oscillator: LFCub<BUFSIZE>,
    filter: Lpf18<BUFSIZE>,
    envelope: LinearASREnvelope<BUFSIZE>,
    balance: PanChan<BUFSIZE, NCHAN>,
    reverb: f32,
    delay: f32,
}

impl<const BUFSIZE: usize, const NCHAN: usize> LFCubSynth<BUFSIZE, NCHAN> {
    pub fn new(sr: f32) -> Self {
        LFCubSynth {
            modulators: Vec::new(),
            oscillator: LFCub::new(440.0, 0.5, sr),
            envelope: LinearASREnvelope::new(0.3, 0.05, 0.1, 0.05, sr),
            filter: Lpf18::new(1500.0, 0.5, 0.1, sr),
            balance: PanChan::new(),
            reverb: 0.0,
            delay: 0.0,
        }
    }
}

impl<const BUFSIZE: usize, const NCHAN: usize> Synth<BUFSIZE, NCHAN>
    for LFCubSynth<BUFSIZE, NCHAN>
{
    fn set_parameter(&mut self, par: SynthParameterLabel, val: &SynthParameterValue) {
        self.oscillator.set_parameter(par, val);
        self.envelope.set_parameter(par, val);
        self.filter.set_parameter(par, val);
        self.balance.set_parameter(par, val);
        match par {
            SynthParameterLabel::ReverbMix => {
                if let SynthParameterValue::ScalarF32(r) = val {
                    self.reverb = *r
                }
            }
            SynthParameterLabel::DelayMix => {
                if let SynthParameterValue::ScalarF32(d) = val {
                    self.delay = *d
                }
            }
            _ => (),
        };
    }

    fn finish(&mut self) {
        self.envelope.finish();
    }

    fn is_finished(&self) -> bool {
        self.envelope.is_finished()
    }

    fn get_next_block(
        &mut self,
        start_sample: usize,
        sample_buffers: &[Vec<f32>],
    ) -> [[f32; BUFSIZE]; NCHAN] {
        let mut out: [f32; BUFSIZE] =
            self.oscillator
                .get_next_block(start_sample, sample_buffers, &self.modulators);
        out = self.envelope.process_block(out, start_sample);
        self.balance.process_block(out)
    }

    fn reverb_level(&self) -> f32 {
        self.reverb
    }

    fn delay_level(&self) -> f32 {
        self.delay
    }
}
