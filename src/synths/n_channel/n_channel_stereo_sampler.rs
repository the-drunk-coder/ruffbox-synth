use crate::building_blocks::envelopes::*;
use crate::building_blocks::filters::*;
use crate::building_blocks::routing::BalChan;
use crate::building_blocks::sampler::StereoSampler;
use crate::building_blocks::SampleBuffer;
use crate::building_blocks::{
    waveshaper::Waveshaper, EnvelopeSegmentInfo, EnvelopeSegmentType, FilterType, Modulator,
    MonoEffect, StereoSource, Synth, SynthParameterLabel, SynthParameterValue,
};

/// a stereo sampler with envelope etc.
/// here we need everything twice ...
pub struct NChannelStereoSampler<const BUFSIZE: usize, const NCHAN: usize> {
    sampler: StereoSampler<BUFSIZE>,
    waveshaper: (Waveshaper<BUFSIZE>, Waveshaper<BUFSIZE>),
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
    pub fn with_bufnum_len(
        bufnum: usize,
        buflen: usize,
        hpf_type: FilterType,
        pf1_type: FilterType,
        pf2_type: FilterType,
        lpf_type: FilterType,
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

        NChannelStereoSampler {
            sampler: StereoSampler::with_bufnum_len(bufnum, buflen, true),
            waveshaper: (Waveshaper::new(), Waveshaper::new()),
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
                    Box::new(ButterworthHpf::new(20.0, order, sr)),
                    Box::new(ButterworthHpf::new(20.0, order, sr)),
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
                    Box::new(ButterworthLpf::new(19000.0, order, sr)),
                    Box::new(ButterworthLpf::new(19000.0, order, sr)),
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
        par: SynthParameterLabel,
        init: f32,
        modulator: Modulator<BUFSIZE>,
    ) {
        self.sampler.set_modulator(par, init, modulator.clone());
        self.hpf.0.set_modulator(par, init, modulator.clone());
        self.hpf.1.set_modulator(par, init, modulator.clone());

        match par {
            SynthParameterLabel::Peak1Frequency => {
                self.peak_eq_1.0.set_modulator(
                    SynthParameterLabel::PeakFrequency,
                    init,
                    modulator.clone(),
                );
                self.peak_eq_1.1.set_modulator(
                    SynthParameterLabel::PeakFrequency,
                    init,
                    modulator.clone(),
                )
            }
            SynthParameterLabel::Peak1Gain => {
                self.peak_eq_1.0.set_modulator(
                    SynthParameterLabel::PeakGain,
                    init,
                    modulator.clone(),
                );
                self.peak_eq_1.1.set_modulator(
                    SynthParameterLabel::PeakGain,
                    init,
                    modulator.clone(),
                );
            }
            SynthParameterLabel::Peak1Bandwidth => {
                self.peak_eq_1.0.set_modulator(
                    SynthParameterLabel::PeakBandwidth,
                    init,
                    modulator.clone(),
                );
                self.peak_eq_1.1.set_modulator(
                    SynthParameterLabel::PeakBandwidth,
                    init,
                    modulator.clone(),
                )
            }
            SynthParameterLabel::Peak2Frequency => {
                self.peak_eq_2.0.set_modulator(
                    SynthParameterLabel::PeakFrequency,
                    init,
                    modulator.clone(),
                );
                self.peak_eq_2.1.set_modulator(
                    SynthParameterLabel::PeakFrequency,
                    init,
                    modulator.clone(),
                );
            }
            SynthParameterLabel::Peak2Gain => {
                self.peak_eq_2.0.set_modulator(
                    SynthParameterLabel::PeakGain,
                    init,
                    modulator.clone(),
                );
                self.peak_eq_2.1.set_modulator(
                    SynthParameterLabel::PeakGain,
                    init,
                    modulator.clone(),
                )
            }
            SynthParameterLabel::Peak2Bandwidth => {
                self.peak_eq_2.0.set_modulator(
                    SynthParameterLabel::PeakBandwidth,
                    init,
                    modulator.clone(),
                );
                self.peak_eq_2.1.set_modulator(
                    SynthParameterLabel::PeakBandwidth,
                    init,
                    modulator.clone(),
                );
            }
            _ => {}
        }

        self.lpf.0.set_modulator(par, init, modulator.clone());
        self.lpf.1.set_modulator(par, init, modulator.clone());
        self.envelope.0.set_modulator(par, init, modulator.clone());
        self.envelope.1.set_modulator(par, init, modulator.clone());
        self.balance.set_modulator(par, init, modulator);
    }

    fn set_parameter(&mut self, par: SynthParameterLabel, val: &SynthParameterValue) {
        self.sampler.set_parameter(par, val);

        self.waveshaper.0.set_parameter(par, val);
        self.waveshaper.1.set_parameter(par, val);

        self.hpf.0.set_parameter(par, val);
        self.hpf.1.set_parameter(par, val);

        match par {
            SynthParameterLabel::Peak1Frequency => {
                self.peak_eq_1
                    .0
                    .set_parameter(SynthParameterLabel::PeakFrequency, val);
                self.peak_eq_1
                    .1
                    .set_parameter(SynthParameterLabel::PeakFrequency, val);
            }
            SynthParameterLabel::Peak1Gain => {
                self.peak_eq_1
                    .0
                    .set_parameter(SynthParameterLabel::PeakGain, val);
                self.peak_eq_1
                    .1
                    .set_parameter(SynthParameterLabel::PeakGain, val);
            }
            SynthParameterLabel::Peak1Bandwidth => {
                self.peak_eq_1
                    .0
                    .set_parameter(SynthParameterLabel::PeakBandwidth, val);
                self.peak_eq_1
                    .1
                    .set_parameter(SynthParameterLabel::PeakBandwidth, val);
            }
            SynthParameterLabel::Peak2Frequency => {
                self.peak_eq_2
                    .0
                    .set_parameter(SynthParameterLabel::PeakFrequency, val);
                self.peak_eq_2
                    .1
                    .set_parameter(SynthParameterLabel::PeakFrequency, val);
            }
            SynthParameterLabel::Peak2Gain => {
                self.peak_eq_2
                    .0
                    .set_parameter(SynthParameterLabel::PeakGain, val);
                self.peak_eq_2
                    .1
                    .set_parameter(SynthParameterLabel::PeakGain, val);
            }
            SynthParameterLabel::Peak2Bandwidth => {
                self.peak_eq_2
                    .0
                    .set_parameter(SynthParameterLabel::PeakBandwidth, val);
                self.peak_eq_2
                    .1
                    .set_parameter(SynthParameterLabel::PeakBandwidth, val);
            }
            _ => {}
        }

        self.lpf.0.set_parameter(par, val);
        self.lpf.1.set_parameter(par, val);
        self.envelope.0.set_parameter(par, val);
        self.envelope.1.set_parameter(par, val);
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

        left = self
            .waveshaper
            .0
            .process_block(left, start_sample, sample_buffers);
        right = self
            .waveshaper
            .1
            .process_block(right, start_sample, sample_buffers);

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
