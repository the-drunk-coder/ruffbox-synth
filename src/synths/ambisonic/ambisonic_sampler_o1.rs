use crate::building_blocks::ambisonics::encoder_o1::EncoderO1;
use crate::building_blocks::bitcrusher::Bitcrusher;
use crate::building_blocks::envelopes::*;
use crate::building_blocks::filters::*;
use crate::building_blocks::sampler::MonoSampler;
use crate::building_blocks::EffectType;
use crate::building_blocks::SynthParameterAddress;
use crate::building_blocks::{
    waveshaper::Waveshaper, FilterType, Modulator, MonoEffect, MonoSource, SampleBuffer, Synth,
    SynthParameterLabel, SynthParameterValue,
};
use crate::synths::SynthDescription;

/// a sampler with envelope etc.
pub struct AmbisonicSamplerO1<const BUFSIZE: usize> {
    sampler: MonoSampler<BUFSIZE>,
    pre_filter_effects: Vec<Box<dyn MonoEffect<BUFSIZE> + Send + Sync>>,
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
    pub fn new(
        desc: SynthDescription,
        bufnum: usize,
        buflen: usize,
        sr: f32,
    ) -> AmbisonicSamplerO1<BUFSIZE> {
        let dur = (buflen as f32 / sr) - 0.0002;

        // fixed filter order for now ...
        let hpf_type = desc.filters.first().unwrap_or(&FilterType::BiquadHpf12dB);
        let pf1_type = desc.filters.get(1).unwrap_or(&FilterType::Dummy);
        let pf2_type = desc.filters.get(2).unwrap_or(&FilterType::Dummy);
        let lpf_type = desc.filters.get(3).unwrap_or(&FilterType::Lpf18);

        let mut pre_filter_effects: Vec<Box<dyn MonoEffect<BUFSIZE> + Sync + Send>> = Vec::new();
        for ef in desc.pre_filter_effects.into_iter() {
            match ef {
                EffectType::Bitcrusher(m) => pre_filter_effects.push(Box::new(Bitcrusher::new(m))),
                EffectType::Waveshaper => pre_filter_effects.push(Box::new(Waveshaper::new())),
            }
        }

        AmbisonicSamplerO1 {
            sampler: MonoSampler::with_bufnum_len(bufnum, buflen, true),
            pre_filter_effects,
            envelope: LinearASREnvelope::new(1.0, 0.0001, dur, 0.0001, sr),
            hpf: match hpf_type {
                FilterType::BiquadHpf12dB => Box::new(BiquadHpf12dB::new(20.0, 0.3, sr)),
                FilterType::BiquadHpf24dB => Box::new(BiquadHpf24dB::new(20.0, 0.3, sr)),
                FilterType::ButterworthHpf(o) => Box::new(ButterworthHpf::new(20.0, *o, sr)),
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
                FilterType::ButterworthLpf(o) => Box::new(ButterworthLpf::new(19000.0, *o, sr)),
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
        par: SynthParameterAddress,
        init: f32,
        modulator: Modulator<BUFSIZE>,
    ) {
        self.sampler
            .set_modulator(par.label, init, modulator.clone());
        self.hpf.set_modulator(par.label, init, modulator.clone());

        for ef in self.pre_filter_effects.iter_mut() {
            ef.set_modulator(par.label, init, modulator.clone());
        }

        match par.label {
            SynthParameterLabel::PeakFrequency
            | SynthParameterLabel::PeakBandwidth
            | SynthParameterLabel::PeakGain => match par.idx {
                Some(1) => self
                    .peak_eq_2
                    .set_modulator(par.label, init, modulator.clone()),
                _ => self
                    .peak_eq_1
                    .set_modulator(par.label, init, modulator.clone()),
            },
            _ => {}
        }

        self.lpf.set_modulator(par.label, init, modulator.clone());
        self.envelope
            .set_modulator(par.label, init, modulator.clone());
        self.encoder.set_modulator(par.label, init, modulator);
    }

    fn set_parameter(&mut self, par: SynthParameterAddress, val: &SynthParameterValue) {
        self.sampler.set_parameter(par.label, val);

        for ef in self.pre_filter_effects.iter_mut() {
            ef.set_parameter(par.label, val);
        }

        self.hpf.set_parameter(par.label, val);

        match par.label {
            SynthParameterLabel::PeakFrequency
            | SynthParameterLabel::PeakBandwidth
            | SynthParameterLabel::PeakGain => match par.idx {
                Some(1) => self.peak_eq_2.set_parameter(par.label, val),
                _ => self.peak_eq_1.set_parameter(par.label, val),
            },
            _ => {}
        }

        self.lpf.set_parameter(par.label, val);
        self.envelope.set_parameter(par.label, val);
        self.encoder.set_parameter(par.label, val);

        match par.label {
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

        for ef in self.pre_filter_effects.iter_mut() {
            out = ef.process_block(out, start_sample, sample_buffers)
        }

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
