use crate::building_blocks::bitcrusher::Bitcrusher;
use crate::building_blocks::delay::MonoDelay;
use crate::building_blocks::envelopes::*;
use crate::building_blocks::filters::BiquadHpf12dB;
use crate::building_blocks::filters::BiquadHpf24dB;
use crate::building_blocks::filters::BiquadLpf12dB;
use crate::building_blocks::filters::BiquadLpf24dB;
use crate::building_blocks::filters::ButterworthHpf;
use crate::building_blocks::filters::ButterworthLpf;
use crate::building_blocks::filters::DummyFilter;
use crate::building_blocks::filters::Lpf18;
use crate::building_blocks::filters::PeakEq;
use crate::building_blocks::oscillators::*;
use crate::building_blocks::routing::PanChan;
use crate::building_blocks::waveshaper::Waveshaper;
use crate::building_blocks::EffectType;
use crate::building_blocks::EnvelopeSegmentInfo;
use crate::building_blocks::EnvelopeSegmentType;
use crate::building_blocks::FilterType;
use crate::building_blocks::OscillatorType;
use crate::building_blocks::Synth;
use crate::building_blocks::SynthParameterAddress;
use crate::building_blocks::{MonoEffect, MonoSource, SynthParameterLabel, SynthParameterValue};
use crate::synths::SynthDescription;

pub struct KarPlusPlus<const BUFSIZE: usize, const NCHAN: usize> {
    source: Box<dyn MonoSource<BUFSIZE> + Sync + Send>,
    source_gain: f32,
    pre_filter_effects: Vec<Box<dyn MonoEffect<BUFSIZE> + Send + Sync>>,
    fb_delay: MonoDelay<BUFSIZE>,
    post_filter: Box<dyn MonoEffect<BUFSIZE> + Sync + Send>,
    envelope: MultiPointEffectEnvelope<BUFSIZE>,
    balance: PanChan<BUFSIZE, NCHAN>,
    reverb: f32,
    delay: f32,
    samplerate: f32,
    burst_len: usize,
}

impl<const BUFSIZE: usize, const NCHAN: usize> KarPlusPlus<BUFSIZE, NCHAN> {
    pub fn new(desc: SynthDescription, samplerate: f32) -> Self {
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

        let envelope = MultiPointEffectEnvelope::new(env_segments, false, samplerate);

        // fixed filter order for now ...
        let post_filter_type = desc.filters.first().unwrap_or(&FilterType::BiquadHpf12dB);
        let delay_filter_type = desc.filters.get(1).unwrap_or(&FilterType::Dummy);

        let mut pre_filter_effects: Vec<Box<dyn MonoEffect<BUFSIZE> + Sync + Send>> = Vec::new();
        for ef in desc.pre_filter_effects.into_iter() {
            match ef {
                EffectType::Bitcrusher(m) => pre_filter_effects.push(Box::new(Bitcrusher::new(m))),
                EffectType::Waveshaper => pre_filter_effects.push(Box::new(Waveshaper::new())),
            }
        }

        KarPlusPlus {
            fb_delay: MonoDelay::with_filter_type(samplerate, *delay_filter_type),
            source: match desc
                .oscillator_types
                .first()
                .unwrap_or(&OscillatorType::WhiteNoise)
            {
                OscillatorType::Sine => Box::new(SineOsc::new(440.0, 0.5, samplerate)),
                OscillatorType::LFTri => Box::new(LFTri::new(440.0, 0.5, samplerate)),
                OscillatorType::LFSquare => Box::new(LFSquare::new(440.0, 0.5, 0.5, samplerate)),
                OscillatorType::LFSaw => Box::new(LFSaw::new(440.0, 0.5, samplerate)),
                OscillatorType::LFRsaw => Box::new(LFRSaw::new(440.0, 0.5, samplerate)),
                OscillatorType::LFCub => Box::new(LFCub::new(440.0, 0.5, samplerate)),
                OscillatorType::FMSquare => Box::new(FMSquare::new(440.0, 0.5, 0.5, samplerate)),
                OscillatorType::FMSaw => Box::new(FMSaw::new(440.0, 0.5, samplerate)),
                OscillatorType::FMTri => Box::new(FMTri::new(440.0, 0.5, samplerate)),
                OscillatorType::WTSaw => Box::new(WTSaw::new(440.0, 0.5, samplerate)),
                OscillatorType::Wavetable => Box::new(Wavetable::new(samplerate)),
                OscillatorType::Wavematrix => Box::new(Wavematrix::new(samplerate)),
                OscillatorType::WhiteNoise => Box::new(WhiteNoise::new(0.2)),
                OscillatorType::BrownNoise => Box::new(BrownNoise::new(0.2, 0.125)),
            },
            pre_filter_effects,
            post_filter: match post_filter_type {
                FilterType::Dummy => Box::new(DummyFilter::new()),
                FilterType::Lpf18 => Box::new(Lpf18::new(1500.0, 0.5, 0.1, samplerate)),
                FilterType::BiquadLpf12dB => Box::new(BiquadLpf12dB::new(1500.0, 0.5, samplerate)),
                FilterType::BiquadLpf24dB => Box::new(BiquadLpf24dB::new(1500.0, 0.5, samplerate)),
                FilterType::BiquadHpf12dB => Box::new(BiquadHpf12dB::new(1500.0, 0.5, samplerate)),
                FilterType::BiquadHpf24dB => Box::new(BiquadHpf24dB::new(1500.0, 0.5, samplerate)),
                FilterType::ButterworthLpf(order) => {
                    Box::new(ButterworthLpf::new(1500.0, *order, samplerate))
                }
                FilterType::ButterworthHpf(order) => {
                    Box::new(ButterworthHpf::new(1500.0, *order, samplerate))
                }
                FilterType::PeakEQ => Box::new(PeakEq::new(1500.0, 100.0, 0.0, samplerate)),
            },
            source_gain: 1.0,
            envelope,
            balance: PanChan::new(),
            reverb: 0.0,
            delay: 0.0,
            samplerate,
            burst_len: 0,
        }
    }
}

impl<const BUFSIZE: usize, const NCHAN: usize> Synth<BUFSIZE, NCHAN>
    for KarPlusPlus<BUFSIZE, NCHAN>
{
    fn set_parameter(&mut self, par: SynthParameterAddress, val: &SynthParameterValue) {
        let SynthParameterAddress { label, idx } = par;

        if let Some(0) = idx {
            self.source.set_parameter(label, val);
        } else {
            match label {
                SynthParameterLabel::PitchFrequency => {
                    if let SynthParameterValue::ScalarF32(f) = val {
                        let del_time_samples = self.samplerate / f;
                        let del_time_s = del_time_samples / self.samplerate;
                        self.burst_len = del_time_samples as usize;
                        self.fb_delay.set_parameter(
                            SynthParameterLabel::DelayTime,
                            &SynthParameterValue::ScalarF32(del_time_s),
                        );
                    }
                }
                SynthParameterLabel::OscillatorAmplitude => {
                    if let SynthParameterValue::ScalarF32(g) = val {
                        self.source_gain = *g;
                    }
                }
                _ => {}
            }
        }

        for ef in self.pre_filter_effects.iter_mut() {
            ef.set_parameter(label, val);
        }

        self.envelope.set_parameter(label, val);
        self.balance.set_parameter(label, val);
        self.fb_delay.set_parameter(label, val);
        self.post_filter.set_parameter(label, val);

        match label {
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

    fn set_modulator(
        &mut self,
        par: SynthParameterAddress,
        init: f32,
        modulator: crate::building_blocks::Modulator<BUFSIZE>,
    ) {
        let SynthParameterAddress { label, idx } = par;

        if let Some(0) = idx {
            self.source.set_modulator(label, init, modulator.clone());
        } else {
            for ef in self.pre_filter_effects.iter_mut() {
                ef.set_modulator(label, init, modulator.clone());
            }

            self.envelope.set_modulator(label, init, modulator.clone());
            self.balance.set_modulator(label, init, modulator.clone());
            self.fb_delay.set_modulator(label, init, modulator.clone());
            self.post_filter
                .set_modulator(label, init, modulator.clone());
        }
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
        sample_buffers: &[crate::building_blocks::SampleBuffer],
    ) -> [[f32; BUFSIZE]; NCHAN] {
        // first, get a random burst block
        let burst_block = if self.burst_len > 0 {
            let mut bb = self.source.get_next_block(start_sample, sample_buffers);
            let block_len = BUFSIZE - start_sample;

            if self.burst_len > block_len {
                self.burst_len -= block_len;
            } else {
                // cut of burst if needed
                for i in (start_sample + self.burst_len)..BUFSIZE {
                    bb[i] = 0.0;
                }
                self.burst_len = 0;
            }
            bb
        } else {
            [0.0; BUFSIZE]
        };

        let mut out = self
            .fb_delay
            .process_block(burst_block, start_sample, sample_buffers);

        for s in start_sample..BUFSIZE {
            out[s] *= self.source_gain;
        }

        for ef in self.pre_filter_effects.iter_mut() {
            out = ef.process_block(out, start_sample, sample_buffers)
        }

        out = self
            .post_filter
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
