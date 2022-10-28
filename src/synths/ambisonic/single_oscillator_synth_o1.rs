use crate::building_blocks::ambisonics::encoder_o1::EncoderO1;
use crate::building_blocks::envelopes::*;
use crate::building_blocks::filters::*;
use crate::building_blocks::oscillators::*;
use crate::building_blocks::{
    FilterType, Modulator, MonoEffect, MonoSource, OscillatorType, Synth, SynthParameterLabel,
    SynthParameterValue,
};

/// a triangle synth with envelope etc.
pub struct SingleOscillatorSynthO1<const BUFSIZE: usize> {
    oscillator: Box<dyn MonoSource<BUFSIZE> + Sync + Send>,
    lp_filter: Box<dyn MonoEffect<BUFSIZE> + Sync + Send>,
    hp_filter: Box<dyn MonoEffect<BUFSIZE> + Sync + Send>,
    envelope: LinearASREnvelope<BUFSIZE>,
    encoder: EncoderO1<BUFSIZE>,
    reverb: f32,
    delay: f32,
}

impl<const BUFSIZE: usize> SingleOscillatorSynthO1<BUFSIZE> {
    pub fn new(
        osc_type: OscillatorType,
        lpf_type: FilterType,
        hpf_type: FilterType,
        sr: f32,
    ) -> Self {
        SingleOscillatorSynthO1 {
            oscillator: match osc_type {
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
            },
            lp_filter: match lpf_type {
                FilterType::Dummy => Box::new(DummyFilter::new()),
                FilterType::Lpf18 => Box::new(Lpf18::new(1500.0, 0.5, 0.1, sr)),
                FilterType::BiquadLpf12dB => Box::new(BiquadLpf12dB::new(1500.0, 0.5, sr)),
                FilterType::BiquadLpf24dB => Box::new(BiquadLpf24dB::new(1500.0, 0.5, sr)),
                FilterType::ButterworthLpf(o) => Box::new(ButterworthLpf::new(1500.0, o, sr)),
                FilterType::PeakEQ => Box::new(PeakEq::new(1500.0, 100.0, 0.0, sr)),
                _ => Box::new(Lpf18::new(1500.0, 0.5, 0.1, sr)),
            },
            hp_filter: match hpf_type {
                FilterType::Dummy => Box::new(DummyFilter::new()),
                FilterType::BiquadHpf12dB => Box::new(BiquadLpf12dB::new(20.0, 0.5, sr)),
                FilterType::BiquadHpf24dB => Box::new(BiquadLpf24dB::new(20.0, 0.5, sr)),
                FilterType::ButterworthHpf(o) => Box::new(ButterworthHpf::new(20.0, o, sr)),
                FilterType::PeakEQ => Box::new(PeakEq::new(20.0, 100.0, 0.0, sr)),
                _ => Box::new(BiquadHpf12dB::new(20.0, 0.5, sr)),
            },
            envelope: LinearASREnvelope::new(0.3, 0.05, 0.1, 0.05, sr),
            encoder: EncoderO1::new(),
            reverb: 0.0,
            delay: 0.0,
        }
    }
}

impl<const BUFSIZE: usize> Synth<BUFSIZE, 4> for SingleOscillatorSynthO1<BUFSIZE> {
    fn set_modulator(
        &mut self,
        par: SynthParameterLabel,
        init: f32,
        modulator: Modulator<BUFSIZE>,
    ) {
        self.oscillator.set_modulator(par, init, modulator.clone());
        self.lp_filter.set_modulator(par, init, modulator.clone());
        self.hp_filter.set_modulator(par, init, modulator.clone());
        self.envelope.set_modulator(par, init, modulator);
        //self.encoder.set_modulator(par, init, modulator);
    }

    fn set_parameter(&mut self, par: SynthParameterLabel, val: &SynthParameterValue) {
        self.oscillator.set_parameter(par, val);
        self.lp_filter.set_parameter(par, val);
        self.hp_filter.set_parameter(par, val);
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
        let mut out: [f32; BUFSIZE] = self.oscillator.get_next_block(start_sample, sample_buffers);
        out = self
            .lp_filter
            .process_block(out, start_sample, sample_buffers);
        out = self
            .hp_filter
            .process_block(out, start_sample, sample_buffers);
        out = self
            .envelope
            .process_block(out, start_sample, sample_buffers);
        self.encoder.process_block(out) // needs the additional info for the modulators
    }

    fn reverb_level(&self) -> f32 {
        self.reverb
    }

    fn delay_level(&self) -> f32 {
        self.delay
    }
}
