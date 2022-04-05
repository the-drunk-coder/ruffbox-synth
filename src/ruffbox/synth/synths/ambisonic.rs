use crate::ruffbox::synth::ambisonics::encoder_o1::EncoderO1;
use crate::ruffbox::synth::envelopes::*;
use crate::ruffbox::synth::filters::*;
use crate::ruffbox::synth::oscillators::*;
use crate::ruffbox::synth::sampler::Sampler;
use crate::ruffbox::synth::Synth;
use crate::ruffbox::synth::SynthParameterLabel;
use crate::ruffbox::synth::*;

/// a sinusoidal synth with envelope etc.
pub struct SineSynthAmbiO1<const BUFSIZE: usize> {
    oscillator: SineOsc<BUFSIZE>,
    envelope: ASREnvelope<BUFSIZE>,
    encoder: EncoderO1<BUFSIZE>,
    reverb: f32,
    delay: f32,
}

impl<const BUFSIZE: usize> SineSynthAmbiO1<BUFSIZE> {
    pub fn new(sr: f32) -> Self {
        SineSynthAmbiO1 {
            oscillator: SineOsc::new(440.0, 0.5, sr),
            envelope: ASREnvelope::new(0.3, 0.05, 0.1, 0.05, sr),
            encoder: EncoderO1::new(),
            reverb: 0.0,
            delay: 0.0,
        }
    }
}

impl<const BUFSIZE: usize> Synth<BUFSIZE, 4> for SineSynthAmbiO1<BUFSIZE> {
    fn set_parameter(&mut self, par: SynthParameterLabel, val: SynthParameterValue) {
        self.oscillator.set_parameter(par, val);
        self.envelope.set_parameter(par, val);
        self.encoder.set_parameter(par, val);
        match par {
            SynthParameterLabel::ReverbMix => {
                if let SynthParameterValue::FloatingPoint(r) = val {
                    self.reverb = r
                }
            }
            SynthParameterLabel::DelayMix => {
                if let SynthParameterValue::FloatingPoint(d) = val {
                    self.delay = d
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
        out = self.envelope.process_block(out, start_sample);
        self.encoder.process_block(out)
    }

    fn reverb_level(&self) -> f32 {
        self.reverb
    }

    fn delay_level(&self) -> f32 {
        self.delay
    }
}

/// a lf triangle synth with envelope etc.
pub struct LFTriSynthAmbiO1<const BUFSIZE: usize> {
    oscillator: LFTri<BUFSIZE>,
    envelope: ASREnvelope<BUFSIZE>,
    encoder: EncoderO1<BUFSIZE>,
    reverb: f32,
    delay: f32,
}

impl<const BUFSIZE: usize> LFTriSynthAmbiO1<BUFSIZE> {
    pub fn new(sr: f32) -> Self {
        LFTriSynthAmbiO1 {
            oscillator: LFTri::new(440.0, 0.5, sr),
            envelope: ASREnvelope::new(0.3, 0.05, 0.1, 0.05, sr),
            encoder: EncoderO1::new(),
            reverb: 0.0,
            delay: 0.0,
        }
    }
}

impl<const BUFSIZE: usize> Synth<BUFSIZE, 4> for LFTriSynthAmbiO1<BUFSIZE> {
    fn set_parameter(&mut self, par: SynthParameterLabel, val: SynthParameterValue) {
        self.oscillator.set_parameter(par, val);
        self.envelope.set_parameter(par, val);
        self.encoder.set_parameter(par, val);
        match par {
            SynthParameterLabel::ReverbMix => {
                if let SynthParameterValue::FloatingPoint(r) = val {
                    self.reverb = r
                }
            }
            SynthParameterLabel::DelayMix => {
                if let SynthParameterValue::FloatingPoint(d) = val {
                    self.delay = d
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
        out = self.envelope.process_block(out, start_sample);
        self.encoder.process_block(out)
    }

    fn reverb_level(&self) -> f32 {
        self.reverb
    }

    fn delay_level(&self) -> f32 {
        self.delay
    }
}

/// a low-frequency sawtooth synth with envelope and lpf18 filter
pub struct LFSawSynthAmbiO1<const BUFSIZE: usize> {
    oscillator: LFSaw<BUFSIZE>,
    filter: Lpf18<BUFSIZE>,
    envelope: ASREnvelope<BUFSIZE>,
    encoder: EncoderO1<BUFSIZE>,
    reverb: f32,
    delay: f32,
}

impl<const BUFSIZE: usize> LFSawSynthAmbiO1<BUFSIZE> {
    pub fn new(sr: f32) -> Self {
        LFSawSynthAmbiO1 {
            oscillator: LFSaw::new(100.0, 0.8, sr),
            filter: Lpf18::new(1500.0, 0.5, 0.1, sr),
            envelope: ASREnvelope::new(1.0, 0.002, 0.02, 0.08, sr),
            encoder: EncoderO1::new(),
            reverb: 0.0,
            delay: 0.0,
        }
    }
}

impl<const BUFSIZE: usize> Synth<BUFSIZE, 4> for LFSawSynthAmbiO1<BUFSIZE> {
    fn set_parameter(&mut self, par: SynthParameterLabel, val: SynthParameterValue) {
        self.oscillator.set_parameter(par, val);
        self.filter.set_parameter(par, val);
        self.envelope.set_parameter(par, val);
        self.encoder.set_parameter(par, val);

        match par {
            SynthParameterLabel::ReverbMix => {
                if let SynthParameterValue::FloatingPoint(r) = val {
                    self.reverb = r
                }
            }
            SynthParameterLabel::DelayMix => {
                if let SynthParameterValue::FloatingPoint(d) = val {
                    self.delay = d
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
        out = self.filter.process_block(out, start_sample);
        out = self.envelope.process_block(out, start_sample);
        self.encoder.process_block(out)
    }

    fn reverb_level(&self) -> f32 {
        self.reverb
    }

    fn delay_level(&self) -> f32 {
        self.delay
    }
}

/// a low-frequency (non-bandlimited) squarewave synth with envelope and lpf18 filter
pub struct LFSquareSynthAmbiO1<const BUFSIZE: usize> {
    oscillator: LFSquare<BUFSIZE>,
    filter: Lpf18<BUFSIZE>,
    envelope: ASREnvelope<BUFSIZE>,
    encoder: EncoderO1<BUFSIZE>,
    reverb: f32,
    delay: f32,
}

impl<const BUFSIZE: usize> LFSquareSynthAmbiO1<BUFSIZE> {
    pub fn new(sr: f32) -> Self {
        LFSquareSynthAmbiO1 {
            oscillator: LFSquare::new(100.0, 0.4, 0.8, sr),
            filter: Lpf18::new(1500.0, 0.5, 0.1, sr),
            envelope: ASREnvelope::new(1.0, 0.002, 0.02, 0.08, sr),
            encoder: EncoderO1::new(),
            reverb: 0.0,
            delay: 0.0,
        }
    }
}

impl<const BUFSIZE: usize> Synth<BUFSIZE, 4> for LFSquareSynthAmbiO1<BUFSIZE> {
    fn set_parameter(&mut self, par: SynthParameterLabel, val: SynthParameterValue) {
        self.oscillator.set_parameter(par, val);
        self.filter.set_parameter(par, val);
        self.envelope.set_parameter(par, val);
        self.encoder.set_parameter(par, val);

        match par {
            SynthParameterLabel::ReverbMix => {
                if let SynthParameterValue::FloatingPoint(r) = val {
                    self.reverb = r
                }
            }
            SynthParameterLabel::DelayMix => {
                if let SynthParameterValue::FloatingPoint(d) = val {
                    self.delay = d
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
        out = self.filter.process_block(out, start_sample);
        out = self.envelope.process_block(out, start_sample);
        self.encoder.process_block(out)
    }

    fn reverb_level(&self) -> f32 {
        self.reverb
    }

    fn delay_level(&self) -> f32 {
        self.delay
    }
}

/// a sampler with envelope etc.
pub struct AmbiSamplerO1<const BUFSIZE: usize> {
    sampler: Sampler<BUFSIZE>,
    envelope: ASREnvelope<BUFSIZE>,
    hpf: BiquadHpf<BUFSIZE>,
    peak_eq: PeakEq<BUFSIZE>,
    lpf: Lpf18<BUFSIZE>,
    encoder: EncoderO1<BUFSIZE>,
    reverb: f32,
    delay: f32,
}

impl<const BUFSIZE: usize> AmbiSamplerO1<BUFSIZE> {
    pub fn with_bufnum_len(bufnum: usize, buflen: usize, sr: f32) -> AmbiSamplerO1<BUFSIZE> {
        let dur = (buflen as f32 / sr) - 0.0002;

        AmbiSamplerO1 {
            sampler: Sampler::with_bufnum_len(bufnum, buflen, true),
            envelope: ASREnvelope::new(1.0, 0.0001, dur, 0.0001, sr),
            hpf: BiquadHpf::new(10.0, 0.01, sr),
            peak_eq: PeakEq::new(700.0, 100.0, 0.0, sr),
            lpf: Lpf18::new(19500.0, 0.01, 0.01, sr),
            encoder: EncoderO1::new(),
            reverb: 0.0,
            delay: 0.0,
        }
    }
}

impl<const BUFSIZE: usize> Synth<BUFSIZE, 4> for AmbiSamplerO1<BUFSIZE> {
    fn set_parameter(&mut self, par: SynthParameterLabel, val: SynthParameterValue) {
        self.sampler.set_parameter(par, val);
        self.hpf.set_parameter(par, val);
        self.peak_eq.set_parameter(par, val);
        self.lpf.set_parameter(par, val);
        self.envelope.set_parameter(par, val);
        self.encoder.set_parameter(par, val);

        match par {
            SynthParameterLabel::ReverbMix => {
                if let SynthParameterValue::FloatingPoint(r) = val {
                    self.reverb = r
                }
            }
            SynthParameterLabel::DelayMix => {
                if let SynthParameterValue::FloatingPoint(d) = val {
                    self.delay = d
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
        let mut out: [f32; BUFSIZE] = self.sampler.get_next_block(start_sample, sample_buffers);
        out = self.hpf.process_block(out, start_sample);
        out = self.peak_eq.process_block(out, start_sample);
        out = self.lpf.process_block(out, start_sample);
        out = self.envelope.process_block(out, start_sample);
        self.encoder.process_block(out)
    }

    fn reverb_level(&self) -> f32 {
        self.reverb
    }

    fn delay_level(&self) -> f32 {
        self.delay
    }
}
