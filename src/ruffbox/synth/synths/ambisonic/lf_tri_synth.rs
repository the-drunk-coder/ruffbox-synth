use crate::ruffbox::synth::ambisonics::encoder_o1::EncoderO1;
use crate::ruffbox::synth::envelopes::*;
use crate::ruffbox::synth::oscillators::*;
use crate::ruffbox::synth::Synth;
use crate::ruffbox::synth::SynthParameterLabel;
use crate::ruffbox::synth::*;

/// a lf triangle synth with envelope etc.
pub struct LFTriSynth<const BUFSIZE: usize> {
    modulators: Vec<Modulator<BUFSIZE>>,
    oscillator: LFTri<BUFSIZE>,
    envelope: LinearASREnvelope<BUFSIZE>,
    encoder: EncoderO1<BUFSIZE>,
    reverb: f32,
    delay: f32,
}

impl<const BUFSIZE: usize> LFTriSynth<BUFSIZE> {
    #[allow(dead_code)]
    pub fn new(sr: f32) -> Self {
        LFTriSynth {
            modulators: Vec::new(),
            oscillator: LFTri::new(440.0, 0.5, sr),
            envelope: LinearASREnvelope::new(0.3, 0.05, 0.1, 0.05, sr),
            encoder: EncoderO1::new(),
            reverb: 0.0,
            delay: 0.0,
        }
    }
}

impl<const BUFSIZE: usize> Synth<BUFSIZE, 4> for LFTriSynth<BUFSIZE> {
    fn set_parameter(&mut self, par: SynthParameterLabel, val: &SynthParameterValue) {
        self.oscillator.set_parameter(par, val);
        self.envelope.set_parameter(par, val);
        self.encoder.set_parameter(par, val);
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
    ) -> [[f32; BUFSIZE]; 4] {
        let mut out: [f32; BUFSIZE] =
            self.oscillator
                .get_next_block(start_sample, sample_buffers, &self.modulators);
        out = self.envelope.process_block(out, start_sample);
        self.encoder.process_block(out)
    }

    fn reverb_level(&self) -> f32 {
        self.reverb
    }

    fn delay_level(&self) -> f32 {
        self.delay
    }
}
