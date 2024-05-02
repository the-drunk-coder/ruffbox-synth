use crate::building_blocks::bitcrusher::Bitcrusher;
use crate::building_blocks::envelopes::*;
use crate::building_blocks::filters::*;
use crate::building_blocks::routing::BalChan;
use crate::building_blocks::sampler::StereoSampler;
use crate::building_blocks::EffectType;
use crate::building_blocks::SampleBuffer;
use crate::building_blocks::SynthParameterAddress;
use crate::building_blocks::{
    waveshaper::Waveshaper, EnvelopeSegmentInfo, EnvelopeSegmentType, FilterType, Modulator,
    MonoEffect, StereoSource, Synth, SynthParameterLabel, SynthParameterValue,
};
use crate::synths::SynthDescription;

/// a stereo sampler with envelope etc.
/// here we need everything twice ...
pub struct NChannelStereoSampler<const BUFSIZE: usize, const NCHAN: usize> {
    sampler: StereoSampler<BUFSIZE>,
    pre_filter_effects: Vec<(
        Box<dyn MonoEffect<BUFSIZE> + Send + Sync>,
        Box<dyn MonoEffect<BUFSIZE> + Send + Sync>,
    )>,
    envelope: (
        MultiPointEffectEnvelope<BUFSIZE>,
        MultiPointEffectEnvelope<BUFSIZE>,
    ),
    hpf: (
        Box<dyn MonoEffect<BUFSIZE> + Send + Sync>,
        Box<dyn MonoEffect<BUFSIZE> + Send + Sync>,
    ),
    peak_eq_1: (
        Box<dyn MonoEffect<BUFSIZE> + Send + Sync>,
        Box<dyn MonoEffect<BUFSIZE> + Send + Sync>,
    ),
    peak_eq_2: (
        Box<dyn MonoEffect<BUFSIZE> + Send + Sync>,
        Box<dyn MonoEffect<BUFSIZE> + Send + Sync>,
    ),
    lpf: (
        Box<dyn MonoEffect<BUFSIZE> + Send + Sync>,
        Box<dyn MonoEffect<BUFSIZE> + Send + Sync>,
    ),
    balance: BalChan<BUFSIZE, NCHAN>,
    reverb: f32,
    delay: f32,
}

impl<const BUFSIZE: usize, const NCHAN: usize> NChannelStereoSampler<BUFSIZE, NCHAN> {
    pub fn new(
        desc: SynthDescription,
        bufnum: usize,
        buflen: usize,
        sr: f32,
    ) -> NChannelStereoSampler<BUFSIZE, NCHAN> {
        let dur = (buflen as f32 / sr) - 0.002;

        // assemble a default ASR envelope ...
        let env_segments = vec![
            EnvelopeSegmentInfo {
                from: 0.0,
                to: 0.6,
                time: 0.001,
                segment_type: EnvelopeSegmentType::Lin,
            },
            EnvelopeSegmentInfo {
                from: 0.6,
                to: 0.6,
                time: dur,
                segment_type: EnvelopeSegmentType::Constant,
            },
            EnvelopeSegmentInfo {
                from: 0.6,
                to: 0.0,
                time: 0.001,
                segment_type: EnvelopeSegmentType::Lin,
            },
        ];
        let env_l = MultiPointEffectEnvelope::new(env_segments.clone(), false, sr);
        let env_r = MultiPointEffectEnvelope::new(env_segments, false, sr);

        // fixed filter order for now ...
        let hpf_type = desc.filters.first().unwrap_or(&FilterType::BiquadHpf12dB);
        let pf1_type = desc.filters.get(1).unwrap_or(&FilterType::Dummy);
        let pf2_type = desc.filters.get(2).unwrap_or(&FilterType::Dummy);
        let lpf_type = desc.filters.get(3).unwrap_or(&FilterType::Lpf18);

        let mut pre_filter_effects: Vec<(
            Box<dyn MonoEffect<BUFSIZE> + Sync + Send>,
            Box<dyn MonoEffect<BUFSIZE> + Sync + Send>,
        )> = Vec::new();
        for ef in desc.pre_filter_effects.into_iter() {
            match ef {
                EffectType::Bitcrusher(m) => pre_filter_effects
                    .push((Box::new(Bitcrusher::new(m)), Box::new(Bitcrusher::new(m)))),
                EffectType::Waveshaper => pre_filter_effects
                    .push((Box::new(Waveshaper::new()), Box::new(Waveshaper::new()))),
            }
        }

        NChannelStereoSampler {
            sampler: StereoSampler::with_bufnum_len(bufnum, buflen, true),
            pre_filter_effects,
            envelope: (env_l, env_r),
            hpf: match hpf_type {
                FilterType::BiquadHpf12dB => (
                    Box::new(BiquadHpf12dB::new(20.0, 0.3, sr)),
                    Box::new(BiquadHpf12dB::new(20.0, 0.3, sr)),
                ),
                FilterType::BiquadHpf24dB => (
                    Box::new(BiquadHpf24dB::new(20.0, 0.3, sr)),
                    Box::new(BiquadHpf24dB::new(20.0, 0.3, sr)),
                ),
                FilterType::ButterworthHpf(order) => (
                    Box::new(ButterworthHpf::new(20.0, *order, sr)),
                    Box::new(ButterworthHpf::new(20.0, *order, sr)),
                ),
                FilterType::Dummy => (Box::new(DummyFilter::new()), Box::new(DummyFilter::new())),
                _ => (
                    Box::new(BiquadHpf12dB::new(20.0, 0.3, sr)),
                    Box::new(BiquadHpf12dB::new(20.0, 0.3, sr)),
                ),
            },
            peak_eq_1: match pf1_type {
                FilterType::PeakEQ => (
                    Box::new(PeakEq::new(700.0, 100.0, 0.0, sr)),
                    Box::new(PeakEq::new(700.0, 100.0, 0.0, sr)),
                ),
                _ => (Box::new(DummyFilter::new()), Box::new(DummyFilter::new())),
            },
            peak_eq_2: match pf2_type {
                FilterType::PeakEQ => (
                    Box::new(PeakEq::new(1500.0, 100.0, 0.0, sr)),
                    Box::new(PeakEq::new(1500.0, 100.0, 0.0, sr)),
                ),
                _ => (Box::new(DummyFilter::new()), Box::new(DummyFilter::new())),
            },
            lpf: match lpf_type {
                FilterType::BiquadLpf12dB => (
                    Box::new(BiquadLpf12dB::new(19000.0, 0.3, sr)),
                    Box::new(BiquadLpf12dB::new(19000.0, 0.3, sr)),
                ),
                FilterType::BiquadLpf24dB => (
                    Box::new(BiquadLpf24dB::new(19000.0, 0.3, sr)),
                    Box::new(BiquadLpf24dB::new(19000.0, 0.3, sr)),
                ),
                FilterType::ButterworthLpf(order) => (
                    Box::new(ButterworthLpf::new(19000.0, *order, sr)),
                    Box::new(ButterworthLpf::new(19000.0, *order, sr)),
                ),
                FilterType::Lpf18 => (
                    Box::new(Lpf18::new(19000.0, 0.1, 0.01, sr)),
                    Box::new(Lpf18::new(19000.0, 0.1, 0.01, sr)),
                ),
                FilterType::Dummy => (Box::new(DummyFilter::new()), Box::new(DummyFilter::new())),
                _ => (
                    Box::new(Lpf18::new(19000.0, 0.1, 0.01, sr)),
                    Box::new(Lpf18::new(19000.0, 0.1, 0.01, sr)),
                ),
            },
            balance: BalChan::new(),
            reverb: 0.0,
            delay: 0.0,
        }
    }
}

impl<const BUFSIZE: usize, const NCHAN: usize> Synth<BUFSIZE, NCHAN>
    for NChannelStereoSampler<BUFSIZE, NCHAN>
{
    fn set_modulator(
        &mut self,
        par: SynthParameterAddress,
        init: f32,
        modulator: Modulator<BUFSIZE>,
    ) {
        self.sampler
            .set_modulator(par.label, init, modulator.clone());
        self.hpf.0.set_modulator(par.label, init, modulator.clone());
        self.hpf.1.set_modulator(par.label, init, modulator.clone());

        for ef in self.pre_filter_effects.iter_mut() {
            ef.0.set_modulator(par.label, init, modulator.clone());
            ef.1.set_modulator(par.label, init, modulator.clone());
        }

        match par.label {
            SynthParameterLabel::PeakFrequency
            | SynthParameterLabel::PeakBandwidth
            | SynthParameterLabel::PeakGain => match par.idx {
                Some(1) => {
                    self.peak_eq_2
                        .0
                        .set_modulator(par.label, init, modulator.clone());
                    self.peak_eq_2
                        .1
                        .set_modulator(par.label, init, modulator.clone());
                }
                _ => {
                    self.peak_eq_1
                        .0
                        .set_modulator(par.label, init, modulator.clone());
                    self.peak_eq_1
                        .1
                        .set_modulator(par.label, init, modulator.clone());
                }
            },

            _ => {}
        }

        self.lpf.0.set_modulator(par.label, init, modulator.clone());
        self.lpf.1.set_modulator(par.label, init, modulator.clone());
        self.envelope
            .0
            .set_modulator(par.label, init, modulator.clone());
        self.envelope
            .1
            .set_modulator(par.label, init, modulator.clone());
        self.balance.set_modulator(par.label, init, modulator);
    }

    fn set_parameter(&mut self, par: SynthParameterAddress, val: &SynthParameterValue) {
        self.sampler.set_parameter(par.label, val);

        for ef in self.pre_filter_effects.iter_mut() {
            ef.0.set_parameter(par.label, val);
            ef.1.set_parameter(par.label, val);
        }

        self.hpf.0.set_parameter(par.label, val);
        self.hpf.1.set_parameter(par.label, val);

        match par.label {
            SynthParameterLabel::PeakFrequency
            | SynthParameterLabel::PeakBandwidth
            | SynthParameterLabel::PeakGain => match par.idx {
                Some(1) => {
                    self.peak_eq_2.0.set_parameter(par.label, val);
                    self.peak_eq_2.1.set_parameter(par.label, val);
                }
                _ => {
                    self.peak_eq_1.0.set_parameter(par.label, val);
                    self.peak_eq_1.1.set_parameter(par.label, val);
                }
            },

            _ => {}
        }

        self.lpf.0.set_parameter(par.label, val);
        self.lpf.1.set_parameter(par.label, val);
        self.envelope.0.set_parameter(par.label, val);
        self.envelope.1.set_parameter(par.label, val);
        self.balance.set_parameter(par.label, val);

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
        self.envelope.0.finish();
        self.envelope.1.finish();
    }

    fn is_finished(&self) -> bool {
        // should always be the same
        self.envelope.0.is_finished() && self.envelope.1.is_finished()
    }

    fn get_next_block(
        &mut self,
        start_sample: usize,
        sample_buffers: &[SampleBuffer],
    ) -> [[f32; BUFSIZE]; NCHAN] {
        let [mut left, mut right]: [[f32; BUFSIZE]; 2] =
            self.sampler.get_next_block(start_sample, sample_buffers);

        for ef in self.pre_filter_effects.iter_mut() {
            left = ef.0.process_block(left, start_sample, sample_buffers);
            right = ef.1.process_block(right, start_sample, sample_buffers);
        }

        left = self.hpf.0.process_block(left, start_sample, sample_buffers);
        right = self
            .hpf
            .1
            .process_block(right, start_sample, sample_buffers);

        left = self
            .peak_eq_1
            .0
            .process_block(left, start_sample, sample_buffers);
        right = self
            .peak_eq_1
            .1
            .process_block(right, start_sample, sample_buffers);

        left = self
            .peak_eq_2
            .0
            .process_block(left, start_sample, sample_buffers);
        right = self
            .peak_eq_2
            .1
            .process_block(right, start_sample, sample_buffers);

        left = self.lpf.0.process_block(left, start_sample, sample_buffers);
        right = self
            .lpf
            .1
            .process_block(right, start_sample, sample_buffers);

        left = self
            .envelope
            .0
            .process_block(left, start_sample, sample_buffers);
        right = self
            .envelope
            .1
            .process_block(right, start_sample, sample_buffers);

        self.balance
            .process_block([left, right], start_sample, sample_buffers) // needs the additional info for the modulators
    }

    fn reverb_level(&self) -> f32 {
        self.reverb
    }

    fn delay_level(&self) -> f32 {
        self.delay
    }
}
