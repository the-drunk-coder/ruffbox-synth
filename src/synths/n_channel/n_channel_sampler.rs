use crate::building_blocks::envelopes::*;
use crate::building_blocks::filters::*;
use crate::building_blocks::routing::PanChan;
use crate::building_blocks::sampler::Sampler;
use crate::building_blocks::{
    EnvelopeSegmentInfo, EnvelopeSegmentType, FilterType, Modulator, MonoEffect, MonoSource, Synth,
    SynthParameterLabel, SynthParameterValue, ValueOrModulator,
};

/// a sampler with envelope etc.
pub struct NChannelSampler<const BUFSIZE: usize, const NCHAN: usize> {
    sampler: Sampler<BUFSIZE>,
    envelope: Box<dyn MonoEffect<BUFSIZE> + Send + Sync>,
    hpf: Box<dyn MonoEffect<BUFSIZE> + Send + Sync>,
    peak_eq_1: Box<dyn MonoEffect<BUFSIZE> + Send + Sync>,
    peak_eq_2: Box<dyn MonoEffect<BUFSIZE> + Send + Sync>,
    lpf: Box<dyn MonoEffect<BUFSIZE> + Send + Sync>,
    balance: PanChan<BUFSIZE, NCHAN>,
    reverb: f32,
    delay: f32,
}

impl<const BUFSIZE: usize, const NCHAN: usize> NChannelSampler<BUFSIZE, NCHAN> {
    pub fn with_bufnum_len(
        bufnum: usize,
        buflen: usize,
        hpf_type: FilterType,
        pf1_type: FilterType,
        pf2_type: FilterType,
        lpf_type: FilterType,
        sr: f32,
    ) -> NChannelSampler<BUFSIZE, NCHAN> {
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
        let env = MultiPointEffectEnvelope::new(env_segments, false, sr);

        NChannelSampler {
            sampler: Sampler::with_bufnum_len(bufnum, buflen, true),
            envelope: Box::new(env),
            hpf: match hpf_type {
                FilterType::BiquadHpf12dB => Box::new(BiquadHpf12dB::new(20.0, 0.3, sr)),
                FilterType::BiquadHpf24dB => Box::new(BiquadHpf24dB::new(20.0, 0.3, sr)),
                FilterType::ButterworthHpf(order) => Box::new(ButterworthHpf::new(20.0, order, sr)),
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
                FilterType::ButterworthLpf(order) => {
                    Box::new(ButterworthLpf::new(19000.0, order, sr))
                }
                FilterType::Lpf18 => Box::new(Lpf18::new(19000.0, 0.1, 0.01, sr)),
                FilterType::Dummy => Box::new(DummyFilter::new()),
                _ => Box::new(Lpf18::new(19000.0, 0.1, 0.01, sr)),
            },
            balance: PanChan::new(),
            reverb: 0.0,
            delay: 0.0,
        }
    }
}

impl<const BUFSIZE: usize, const NCHAN: usize> Synth<BUFSIZE, NCHAN>
    for NChannelSampler<BUFSIZE, NCHAN>
{
    fn set_param_or_modulator(
        &mut self,
        par: SynthParameterLabel,
        val_or_mod: ValueOrModulator<BUFSIZE>,
    ) {
        match val_or_mod {
            ValueOrModulator::Val(val) => self.set_parameter(par, &val),
            ValueOrModulator::Mod(init, modulator) => self.set_modulator(par, init, modulator),
        }
    }

    fn set_modulator(
        &mut self,
        par: SynthParameterLabel,
        init: f32,
        modulator: Modulator<BUFSIZE>,
    ) {
        self.sampler.set_modulator(par, init, modulator.clone());
        self.hpf.set_modulator(par, init, modulator.clone());

        match par {
            SynthParameterLabel::Peak1Frequency => self.peak_eq_1.set_modulator(
                SynthParameterLabel::PeakFrequency,
                init,
                modulator.clone(),
            ),
            SynthParameterLabel::Peak1Gain => {
                self.peak_eq_1
                    .set_modulator(SynthParameterLabel::PeakGain, init, modulator.clone())
            }
            SynthParameterLabel::Peak1Bandwidth => self.peak_eq_1.set_modulator(
                SynthParameterLabel::PeakBandwidth,
                init,
                modulator.clone(),
            ),
            SynthParameterLabel::Peak2Frequency => self.peak_eq_2.set_modulator(
                SynthParameterLabel::PeakFrequency,
                init,
                modulator.clone(),
            ),
            SynthParameterLabel::Peak2Gain => {
                self.peak_eq_2
                    .set_modulator(SynthParameterLabel::PeakGain, init, modulator.clone())
            }
            SynthParameterLabel::Peak2Bandwidth => self.peak_eq_2.set_modulator(
                SynthParameterLabel::PeakBandwidth,
                init,
                modulator.clone(),
            ),
            _ => {}
        }

        self.lpf.set_modulator(par, init, modulator.clone());
        self.envelope.set_modulator(par, init, modulator.clone());
        self.balance.set_modulator(par, init, modulator);
    }

    fn set_parameter(&mut self, par: SynthParameterLabel, val: &SynthParameterValue) {
        self.sampler.set_parameter(par, val);
        self.hpf.set_parameter(par, val);

        match par {
            SynthParameterLabel::Peak1Frequency => self
                .peak_eq_1
                .set_parameter(SynthParameterLabel::PeakFrequency, val),
            SynthParameterLabel::Peak1Gain => self
                .peak_eq_1
                .set_parameter(SynthParameterLabel::PeakGain, val),
            SynthParameterLabel::Peak1Bandwidth => self
                .peak_eq_1
                .set_parameter(SynthParameterLabel::PeakBandwidth, val),
            SynthParameterLabel::Peak2Frequency => self
                .peak_eq_2
                .set_parameter(SynthParameterLabel::PeakFrequency, val),
            SynthParameterLabel::Peak2Gain => self
                .peak_eq_2
                .set_parameter(SynthParameterLabel::PeakGain, val),
            SynthParameterLabel::Peak2Bandwidth => self
                .peak_eq_2
                .set_parameter(SynthParameterLabel::PeakBandwidth, val),
            _ => {}
        }

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
        let mut out: [f32; BUFSIZE] = self.sampler.get_next_block(start_sample, sample_buffers);
        out = self.hpf.process_block(out, start_sample, sample_buffers);
        out = self
            .peak_eq_1
            .process_block(out, start_sample, sample_buffers);
        out = self
            .peak_eq_2
            .process_block(out, start_sample, sample_buffers);
        out = self.lpf.process_block(out, start_sample, sample_buffers);
        out = self
            .envelope
            .process_block(out, start_sample, sample_buffers);
        self.balance
            .process_block(out, start_sample, sample_buffers) // needs the additional info for the modulators
    }

    fn reverb_level(&self) -> f32 {
        self.reverb
    }

    fn delay_level(&self) -> f32 {
        self.delay
    }
}
