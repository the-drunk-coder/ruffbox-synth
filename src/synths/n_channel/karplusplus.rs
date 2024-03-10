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
use crate::building_blocks::EnvelopeSegmentInfo;
use crate::building_blocks::EnvelopeSegmentType;
use crate::building_blocks::FilterType;
use crate::building_blocks::OscillatorType;
use crate::building_blocks::Synth;
use crate::building_blocks::SynthParameterAddress;
use crate::building_blocks::{MonoEffect, MonoSource, SynthParameterLabel, SynthParameterValue};

pub struct KarPlusPlus<const BUFSIZE: usize, const NCHAN: usize> {
    source: Box<dyn MonoSource<BUFSIZE> + Sync + Send>,
    fb_delay: MonoDelay<BUFSIZE>,
    waveshaper: Waveshaper<BUFSIZE>,
    post_filter: Box<dyn MonoEffect<BUFSIZE> + Sync + Send>,
    envelope: MultiPointEffectEnvelope<BUFSIZE>,
    balance: PanChan<BUFSIZE, NCHAN>,
    reverb: f32,
    delay: f32,
    samplerate: f32,
    burst_len: usize,
}

impl<const BUFSIZE: usize, const NCHAN: usize> KarPlusPlus<BUFSIZE, NCHAN> {
    pub fn new(
        source_type: OscillatorType,
        delay_filter_type: FilterType,
        post_filter_type: FilterType,
        samplerate: f32,
    ) -> Self {
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

        KarPlusPlus {
            fb_delay: MonoDelay::with_filter_type(samplerate, delay_filter_type),
            source: match source_type {
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
            post_filter: match post_filter_type {
                FilterType::Dummy => Box::new(DummyFilter::new()),
                FilterType::Lpf18 => Box::new(Lpf18::new(1500.0, 0.5, 0.1, samplerate)),
                FilterType::BiquadLpf12dB => Box::new(BiquadLpf12dB::new(1500.0, 0.5, samplerate)),
                FilterType::BiquadLpf24dB => Box::new(BiquadLpf24dB::new(1500.0, 0.5, samplerate)),
                FilterType::BiquadHpf12dB => Box::new(BiquadHpf12dB::new(1500.0, 0.5, samplerate)),
                FilterType::BiquadHpf24dB => Box::new(BiquadHpf24dB::new(1500.0, 0.5, samplerate)),
                FilterType::ButterworthLpf(order) => {
                    Box::new(ButterworthLpf::new(1500.0, order, samplerate))
                }
                FilterType::ButterworthHpf(order) => {
                    Box::new(ButterworthHpf::new(1500.0, order, samplerate))
                }
                FilterType::PeakEQ => Box::new(PeakEq::new(1500.0, 100.0, 0.0, samplerate)),
            },
            waveshaper: Waveshaper::new(),
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
        match par {
            SynthParameterAddress { label, idx: _ } => match label {
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

                _ => {}
            },
        }

        self.waveshaper.set_parameter(par.label, val);
        self.envelope.set_parameter(par.label, val);
        self.balance.set_parameter(par.label, val);
        self.fb_delay.set_parameter(par.label, val);
        self.post_filter.set_parameter(par.label, val);

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

    fn set_modulator(
        &mut self,
        _: SynthParameterAddress,
        _: f32,
        _: crate::building_blocks::Modulator<BUFSIZE>,
    ) {
        // no modulators so far
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
                for i in self.burst_len..BUFSIZE {
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

        out = self
            .waveshaper
            .process_block(out, start_sample, sample_buffers);

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

    fn set_param_or_modulator(
        &mut self,
        par: SynthParameterAddress,
        val_or_mod: crate::building_blocks::ValueOrModulator<BUFSIZE>,
    ) {
        match val_or_mod {
            crate::building_blocks::ValueOrModulator::Val(val) => self.set_parameter(par, &val),
            crate::building_blocks::ValueOrModulator::Mod(init, modulator) => {
                self.set_modulator(par, init, modulator)
            }
        }
    }
}
