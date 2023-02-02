use crate::building_blocks::ambisonics::encoder_o1::EncoderO1;
use crate::building_blocks::envelopes::*;
use crate::building_blocks::filters::*;
use crate::building_blocks::sampler::MonoSampler;
use crate::building_blocks::{
    FilterType, Modulator, MonoEffect, MonoSource, SampleBuffer, Synth, SynthParameterLabel,
    SynthParameterValue,
};

/// a sampler with envelope etc.
pub struct AmbisonicSamplerO1<const BUFSIZE: usize> {
    sampler: MonoSampler<BUFSIZE>,
    envelope: LinearASREnvelope<BUFSIZE>,
    hpf: Box<dyn MonoEffect<BUFSIZE> + Send + Sync>,
    peak_eq_1: Box<dyn MonoEffect<BUFSIZE> + Send + Sync>,
    peak_eq_2: Box<dyn MonoEffect<BUFSIZE> + Send + Sync>,
    lpf: Box<dyn MonoEffect<BUFSIZE> + Send + Sync>,
    encoder: EncoderO1<BUFSIZE>,
    reverb: f32,
    delay: f32,
}

impl<const BUFSIZE: usize> AmbisonicSamplerO1<BUFSIZE> {
    pub fn with_bufnum_len(
        bufnum: usize,
        buflen: usize,
        hpf_type: FilterType,
        pf1_type: FilterType,
        pf2_type: FilterType,
        lpf_type: FilterType,
        sr: f32,
    ) -> AmbisonicSamplerO1<BUFSIZE> {
        let dur = (buflen as f32 / sr) - 0.0002;

        AmbisonicSamplerO1 {
            sampler: MonoSampler::with_bufnum_len(bufnum, buflen, true),
            envelope: LinearASREnvelope::new(1.0, 0.0001, dur, 0.0001, sr),
            hpf: match hpf_type {
                FilterType::BiquadHpf12dB => Box::new(BiquadHpf12dB::new(20.0, 0.3, sr)),
                FilterType::BiquadHpf24dB => Box::new(BiquadHpf24dB::new(20.0, 0.3, sr)),
                FilterType::ButterworthHpf(o) => Box::new(ButterworthHpf::new(20.0, o, sr)),
                FilterType::Dummy => Box::new(DummyFilter::new()),
                _ => Box::new(BiquadHpf12dB::new(20.0, 0.3, sr)),
            },
            peak_eq_1: match pf1_type {
                FilterType::PeakEQ => Box::new(PeakEq::new(700.0, 100.0, 0.0, sr)),
                _ => Box::new(DummyFilter::new()),
            },
            peak_eq_2: match pf2_type {
                FilterType::PeakEQ => Box::new(PeakEq::new(1500.0, 100.0, 0.0, sr)),
                _ => Box::new(DummyFilter::new()),
            },
            lpf: match lpf_type {
                FilterType::BiquadLpf12dB => Box::new(BiquadLpf12dB::new(19000.0, 0.3, sr)),
                FilterType::BiquadLpf24dB => Box::new(BiquadLpf24dB::new(19000.0, 0.3, sr)),
                FilterType::ButterworthLpf(o) => Box::new(ButterworthLpf::new(19000.0, o, sr)),
                FilterType::Lpf18 => Box::new(Lpf18::new(19000.0, 0.1, 0.01, sr)),
                FilterType::Dummy => Box::new(DummyFilter::new()),
                _ => Box::new(Lpf18::new(19000.0, 0.1, 0.01, sr)),
            },
            encoder: EncoderO1::new(),
            reverb: 0.0,
            delay: 0.0,
        }
    }
}

impl<const BUFSIZE: usize> Synth<BUFSIZE, 4> for AmbisonicSamplerO1<BUFSIZE> {
    fn set_modulator(
        &mut self,
        par: SynthParameterLabel,
        init: f32,
        modulator: Modulator<BUFSIZE>,
    ) {
        self.sampler.set_modulator(par, init, modulator.clone());
        self.hpf.set_modulator(par, init, modulator.clone());
        self.peak_eq_1.set_modulator(par, init, modulator.clone());
        self.peak_eq_2.set_modulator(par, init, modulator.clone());
        self.lpf.set_modulator(par, init, modulator.clone());
        self.envelope.set_modulator(par, init, modulator.clone());
        self.encoder.set_modulator(par, init, modulator);
    }
    fn set_parameter(&mut self, par: SynthParameterLabel, val: &SynthParameterValue) {
        self.sampler.set_parameter(par, val);
        self.hpf.set_parameter(par, val);
        self.peak_eq_1.set_parameter(par, val);
        self.peak_eq_2.set_parameter(par, val);
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
        sample_buffers: &[SampleBuffer],
    ) -> [[f32; BUFSIZE]; 4] {
        let mut out: [f32; BUFSIZE] = self.sampler.get_next_block(start_sample, sample_buffers);
        out = self.hpf.process_block(out, start_sample, sample_buffers);
        out = self
            .peak_eq_1
            .process_block(out, start_sample, sample_buffers);
        out = self
            .peak_eq_1
            .process_block(out, start_sample, sample_buffers);
        out = self.lpf.process_block(out, start_sample, sample_buffers);
        out = self
            .envelope
            .process_block(out, start_sample, sample_buffers);
        self.encoder
            .process_block(out, start_sample, sample_buffers)
    }

    fn reverb_level(&self) -> f32 {
        self.reverb
    }

    fn delay_level(&self) -> f32 {
        self.delay
    }
}
