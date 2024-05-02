use crate::building_blocks::bitcrusher::Bitcrusher;
use crate::building_blocks::envelopes::*;
use crate::building_blocks::filters::*;
use crate::building_blocks::oscillators::*;
use crate::building_blocks::routing::PanChan;
use crate::building_blocks::EffectType;
use crate::building_blocks::SynthParameterAddress;
use crate::building_blocks::{
    waveshaper::Waveshaper, EnvelopeSegmentInfo, EnvelopeSegmentType, FilterType, Modulator,
    MonoEffect, MonoSource, OscillatorType, SampleBuffer, Synth, SynthParameterLabel,
    SynthParameterValue,
};
use crate::synths::SynthDescription;

/// a triangle synth with envelope etc.
pub struct SingleOscillatorSynth<const BUFSIZE: usize, const NCHAN: usize> {
    oscillator: Box<dyn MonoSource<BUFSIZE> + Sync + Send>,
    pre_filter_effects: Vec<Box<dyn MonoEffect<BUFSIZE> + Send + Sync>>,
    lp_filter: Box<dyn MonoEffect<BUFSIZE> + Sync + Send>,
    hp_filter: Box<dyn MonoEffect<BUFSIZE> + Sync + Send>,
    envelope: MultiPointEffectEnvelope<BUFSIZE>,
    balance: PanChan<BUFSIZE, NCHAN>,
    reverb: f32,
    delay: f32,
}

impl<const BUFSIZE: usize, const NCHAN: usize> SingleOscillatorSynth<BUFSIZE, NCHAN> {
    pub fn new(desc: SynthDescription, sr: f32) -> Self {
        // assemble a default ASR envelope ...
        let env_segments = vec![
            EnvelopeSegmentInfo {
                from: 0.0,
                to: 0.6,
                time: 0.007,
                segment_type: EnvelopeSegmentType::Lin,
            },
            EnvelopeSegmentInfo {
                from: 0.6,
                to: 0.6,
                time: 0.1,
                segment_type: EnvelopeSegmentType::Constant,
            },
            EnvelopeSegmentInfo {
                from: 0.6,
                to: 0.0,
                time: 0.001,
                segment_type: EnvelopeSegmentType::Lin,
            },
        ];

        let envelope = MultiPointEffectEnvelope::new(env_segments, false, sr);

        let hpf_type = desc.filters.first().unwrap_or(&FilterType::Lpf18);
        let lpf_type = desc.filters.get(1).unwrap_or(&FilterType::BiquadHpf12dB);

        let mut pre_filter_effects: Vec<Box<dyn MonoEffect<BUFSIZE> + Sync + Send>> = Vec::new();
        for ef in desc.pre_filter_effects.into_iter() {
            match ef {
                EffectType::Bitcrusher(m) => pre_filter_effects.push(Box::new(Bitcrusher::new(m))),
                EffectType::Waveshaper => pre_filter_effects.push(Box::new(Waveshaper::new())),
            }
        }

        SingleOscillatorSynth {
            oscillator: match desc
                .oscillator_types
                .first()
                .unwrap_or(&OscillatorType::Sine)
            {
                OscillatorType::Sine => Box::new(SineOsc::new(440.0, 0.5, sr)),
                OscillatorType::LFTri => Box::new(LFTri::new(440.0, 0.5, sr)),
                OscillatorType::LFSquare => Box::new(LFSquare::new(440.0, 0.5, 0.5, sr)),
                OscillatorType::LFSaw => Box::new(LFSaw::new(440.0, 0.5, sr)),
                OscillatorType::LFRsaw => Box::new(LFRSaw::new(440.0, 0.5, sr)),
                OscillatorType::LFCub => Box::new(LFCub::new(440.0, 0.5, sr)),
                OscillatorType::FMSquare => Box::new(FMSquare::new(440.0, 0.5, 0.5, sr)),
                OscillatorType::FMSaw => Box::new(FMSaw::new(440.0, 0.5, sr)),
                OscillatorType::FMTri => Box::new(FMTri::new(440.0, 0.5, sr)),
                OscillatorType::WTSaw => Box::new(WTSaw::new(440.0, 0.5, sr)),
                OscillatorType::Wavetable => Box::new(Wavetable::new(sr)),
                OscillatorType::Wavematrix => Box::new(Wavematrix::new(sr)),
                OscillatorType::WhiteNoise => Box::new(WhiteNoise::new(0.2)),
                OscillatorType::BrownNoise => Box::new(BrownNoise::new(0.2, 0.125)),
            },
            pre_filter_effects,
            lp_filter: match lpf_type {
                FilterType::Dummy => Box::new(DummyFilter::new()),
                FilterType::Lpf18 => Box::new(Lpf18::new(1500.0, 0.5, 0.1, sr)),
                FilterType::BiquadLpf12dB => Box::new(BiquadLpf12dB::new(1500.0, 0.5, sr)),
                FilterType::BiquadLpf24dB => Box::new(BiquadLpf24dB::new(1500.0, 0.5, sr)),
                FilterType::ButterworthLpf(order) => {
                    Box::new(ButterworthLpf::new(1500.0, *order, sr))
                }
                FilterType::PeakEQ => Box::new(PeakEq::new(1500.0, 100.0, 0.0, sr)),
                _ => Box::new(Lpf18::new(1500.0, 0.5, 0.1, sr)),
            },
            hp_filter: match hpf_type {
                FilterType::Dummy => Box::new(DummyFilter::new()),
                FilterType::BiquadHpf12dB => Box::new(BiquadHpf12dB::new(20.0, 0.5, sr)),
                FilterType::BiquadHpf24dB => Box::new(BiquadHpf24dB::new(20.0, 0.5, sr)),
                FilterType::ButterworthHpf(order) => {
                    Box::new(ButterworthHpf::new(20.0, *order, sr))
                }
                FilterType::PeakEQ => Box::new(PeakEq::new(500.0, 100.0, 0.0, sr)),
                _ => Box::new(BiquadHpf12dB::new(1500.0, 0.5, sr)),
            },
            envelope,
            balance: PanChan::new(),
            reverb: 0.0,
            delay: 0.0,
        }
    }
}

impl<const BUFSIZE: usize, const NCHAN: usize> Synth<BUFSIZE, NCHAN>
    for SingleOscillatorSynth<BUFSIZE, NCHAN>
{
    fn set_modulator(
        &mut self,
        par: SynthParameterAddress,
        init: f32,
        modulator: Modulator<BUFSIZE>,
    ) {
        self.oscillator
            .set_modulator(par.label, init, modulator.clone());

        for ef in self.pre_filter_effects.iter_mut() {
            ef.set_modulator(par.label, init, modulator.clone());
        }

        self.lp_filter
            .set_modulator(par.label, init, modulator.clone());
        self.hp_filter
            .set_modulator(par.label, init, modulator.clone());
        self.envelope
            .set_modulator(par.label, init, modulator.clone());
        self.balance.set_modulator(par.label, init, modulator);
    }

    fn set_parameter(&mut self, par: SynthParameterAddress, val: &SynthParameterValue) {
        self.oscillator.set_parameter(par.label, val);

        for ef in self.pre_filter_effects.iter_mut() {
            ef.set_parameter(par.label, val);
        }

        self.lp_filter.set_parameter(par.label, val);
        self.hp_filter.set_parameter(par.label, val);
        self.envelope.set_parameter(par.label, val);
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
        self.envelope.finish();
    }

    fn is_finished(&self) -> bool {
        self.envelope.is_finished()
    }

    fn get_next_block(
        &mut self,
        start_sample: usize,
        sample_buffers: &[SampleBuffer],
    ) -> [[f32; BUFSIZE]; NCHAN] {
        let mut out: [f32; BUFSIZE] = self.oscillator.get_next_block(start_sample, sample_buffers);
        for ef in self.pre_filter_effects.iter_mut() {
            out = ef.process_block(out, start_sample, sample_buffers)
        }
        out = self
            .lp_filter
            .process_block(out, start_sample, sample_buffers);
        out = self
            .hp_filter
            .process_block(out, start_sample, sample_buffers);
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
