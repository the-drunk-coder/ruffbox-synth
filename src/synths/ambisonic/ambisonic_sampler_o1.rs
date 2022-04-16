use crate::building_blocks::ambisonics::encoder_o1::EncoderO1;
use crate::building_blocks::envelopes::*;
use crate::building_blocks::filters::*;
use crate::building_blocks::sampler::Sampler;
use crate::building_blocks::{
    Modulator, MonoEffect, MonoSource, Synth, SynthParameterLabel, SynthParameterValue,
};

/// a sampler with envelope etc.
pub struct AmbisonicSamplerO1<const BUFSIZE: usize> {
    modulators: Vec<Modulator<BUFSIZE>>,
    sampler: Sampler<BUFSIZE>,
    envelope: LinearASREnvelope<BUFSIZE>,
    hpf: BiquadHpf<BUFSIZE>,
    peak_eq: PeakEq<BUFSIZE>,
    lpf: Lpf18<BUFSIZE>,
    encoder: EncoderO1<BUFSIZE>,
    reverb: f32,
    delay: f32,
}

impl<const BUFSIZE: usize> AmbisonicSamplerO1<BUFSIZE> {
    pub fn with_bufnum_len(bufnum: usize, buflen: usize, sr: f32) -> AmbisonicSamplerO1<BUFSIZE> {
        let dur = (buflen as f32 / sr) - 0.0002;

        AmbisonicSamplerO1 {
            modulators: Vec::new(),
            sampler: Sampler::with_bufnum_len(bufnum, buflen, true),
            envelope: LinearASREnvelope::new(1.0, 0.0001, dur, 0.0001, sr),
            hpf: BiquadHpf::new(10.0, 0.01, sr),
            peak_eq: PeakEq::new(700.0, 100.0, 0.0, sr),
            lpf: Lpf18::new(19500.0, 0.01, 0.01, sr),
            encoder: EncoderO1::new(),
            reverb: 0.0,
            delay: 0.0,
        }
    }
}

impl<const BUFSIZE: usize> Synth<BUFSIZE, 4> for AmbisonicSamplerO1<BUFSIZE> {
    fn set_parameter(&mut self, par: SynthParameterLabel, val: &SynthParameterValue) {
        self.sampler.set_parameter(par, val);
        self.hpf.set_parameter(par, val);
        self.peak_eq.set_parameter(par, val);
        self.lpf.set_parameter(par, val);
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
            self.sampler
                .get_next_block(start_sample, sample_buffers, &self.modulators);
        out = self.hpf.process_block(out, start_sample, &self.modulators);
        out = self
            .peak_eq
            .process_block(out, start_sample, &self.modulators);
        out = self.lpf.process_block(out, start_sample, &self.modulators);
        out = self
            .envelope
            .process_block(out, start_sample, &self.modulators);
        self.encoder.process_block(out)
    }

    fn reverb_level(&self) -> f32 {
        self.reverb
    }

    fn delay_level(&self) -> f32 {
        self.delay
    }
}
