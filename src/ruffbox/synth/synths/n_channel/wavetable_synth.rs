use crate::ruffbox::synth::envelopes::*;
use crate::ruffbox::synth::filters::*;
use crate::ruffbox::synth::oscillators::wavetable::Wavetable;
use crate::ruffbox::synth::routing::PanChan;
use crate::ruffbox::synth::Synth;
use crate::ruffbox::synth::*;
use crate::ruffbox::synth::{SynthParameterLabel, SynthParameterValue};

/// a simple wavetable synth with envelope etc.
pub struct WavetableSynth<const BUFSIZE: usize, const NCHAN: usize> {
    wavetable: Wavetable<BUFSIZE>,
    envelope: LinearASREnvelope<BUFSIZE>,
    hpf: BiquadHpf<BUFSIZE>,
    lpf: Lpf18<BUFSIZE>,
    balance: PanChan<BUFSIZE, NCHAN>,
    reverb: f32,
    delay: f32,
}

impl<const BUFSIZE: usize, const NCHAN: usize> WavetableSynth<BUFSIZE, NCHAN> {
    pub fn new(sr: f32) -> WavetableSynth<BUFSIZE, NCHAN> {
        WavetableSynth {
            wavetable: Wavetable::new(sr),
            envelope: LinearASREnvelope::new(1.0, 0.0001, 0.1, 0.0001, sr),
            hpf: BiquadHpf::new(20.0, 0.3, sr),
            lpf: Lpf18::new(19500.0, 0.01, 0.01, sr),
            balance: PanChan::new(),
            reverb: 0.0,
            delay: 0.0,
        }
    }
}

impl<const BUFSIZE: usize, const NCHAN: usize> Synth<BUFSIZE, NCHAN>
    for WavetableSynth<BUFSIZE, NCHAN>
{
    fn set_parameter(&mut self, par: SynthParameterLabel, val: &SynthParameterValue) {
        self.wavetable.set_parameter(par, val);
        self.hpf.set_parameter(par, val);
        self.lpf.set_parameter(par, val);
        self.envelope.set_parameter(par, val);
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
        let mut out: [f32; BUFSIZE] = self.wavetable.get_next_block(start_sample, sample_buffers);
        out = self.hpf.process_block(out, start_sample);
        out = self.lpf.process_block(out, start_sample);
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
