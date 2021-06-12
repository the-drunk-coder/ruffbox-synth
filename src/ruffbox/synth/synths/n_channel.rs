use crate::ruffbox::synth::envelopes::*;
use crate::ruffbox::synth::filters::*;
use crate::ruffbox::synth::oscillators::*;
use crate::ruffbox::synth::routing::PanChan;
use crate::ruffbox::synth::sampler::Sampler;
use crate::ruffbox::synth::Synth;
use crate::ruffbox::synth::SynthParameter;
use crate::ruffbox::synth::*;

/// a sinusoidal synth with envelope etc.
pub struct SineSynth<const BUFSIZE: usize, const NCHAN: usize> {
    oscillator: SineOsc<BUFSIZE>,
    envelope: ASREnvelope<BUFSIZE>,
    balance: PanChan<BUFSIZE, NCHAN>,
    reverb: f32,
    delay: f32,
}

impl<const BUFSIZE: usize, const NCHAN: usize> SineSynth<BUFSIZE, NCHAN> {
    pub fn new(sr: f32) -> Self {
        SineSynth {
            oscillator: SineOsc::new(440.0, 0.5, sr),
            envelope: ASREnvelope::new(sr, 0.3, 0.05, 0.1, 0.05),
            balance: PanChan::new(),
            reverb: 0.0,
            delay: 0.0,
        }
    }
}

impl<const BUFSIZE: usize, const NCHAN: usize> Synth<BUFSIZE, NCHAN> for SineSynth<BUFSIZE, NCHAN> {
    fn set_parameter(&mut self, par: SynthParameter, val: f32) {
        self.oscillator.set_parameter(par, val);
        self.envelope.set_parameter(par, val);
        self.balance.set_parameter(par, val);
        match par {
            SynthParameter::ReverbMix => self.reverb = val,
            SynthParameter::DelayMix => self.delay = val,
            _ => (),
        };
    }

    fn finish(&mut self) {
        self.envelope.finish();
    }

    fn is_finished(&self) -> bool {
        self.envelope.is_finished()
    }

    fn get_next_block(&mut self, start_sample: usize, sample_buffers: &Vec<Vec<f32>>) -> [[f32; BUFSIZE]; NCHAN] {
        let mut out: [f32; BUFSIZE] = self.oscillator.get_next_block(start_sample, sample_buffers);
        out = self.envelope.process_block(out, start_sample);
        self.balance.process_block(out)
    }

    fn reverb_level(&self) -> f32 {
        self.reverb
    }

    fn delay_level(&self) -> f32 {
        self.delay
    }
}

/// a cubic sine approximation synth with envelope etc.
pub struct LFCubSynth<const BUFSIZE: usize, const NCHAN: usize> {
    oscillator: LFCub<BUFSIZE>,
    filter: Lpf18<BUFSIZE>,
    envelope: ASREnvelope<BUFSIZE>,
    balance: PanChan<BUFSIZE, NCHAN>,
    reverb: f32,
    delay: f32,
}

impl<const BUFSIZE: usize, const NCHAN: usize> LFCubSynth<BUFSIZE, NCHAN> {
    pub fn new(sr: f32) -> Self {
        LFCubSynth {
            oscillator: LFCub::new(440.0, 0.5, sr),
            envelope: ASREnvelope::new(sr, 0.3, 0.05, 0.1, 0.05),
            filter: Lpf18::new(1500.0, 0.5, 0.1, sr),
            balance: PanChan::new(),
            reverb: 0.0,
            delay: 0.0,
        }
    }
}

impl<const BUFSIZE: usize, const NCHAN: usize> Synth<BUFSIZE, NCHAN>
    for LFCubSynth<BUFSIZE, NCHAN>
{
    fn set_parameter(&mut self, par: SynthParameter, val: f32) {
        self.oscillator.set_parameter(par, val);
        self.envelope.set_parameter(par, val);
        self.filter.set_parameter(par, val);
        self.balance.set_parameter(par, val);
        match par {
            SynthParameter::ReverbMix => self.reverb = val,
            SynthParameter::DelayMix => self.delay = val,
            _ => (),
        };
    }

    fn finish(&mut self) {
        self.envelope.finish();
    }

    fn is_finished(&self) -> bool {
        self.envelope.is_finished()
    }

    fn get_next_block(&mut self, start_sample: usize, sample_buffers: &Vec<Vec<f32>>) -> [[f32; BUFSIZE]; NCHAN] {
        let mut out: [f32; BUFSIZE] = self.oscillator.get_next_block(start_sample, sample_buffers);
        out = self.envelope.process_block(out, start_sample);
        self.balance.process_block(out)
    }

    fn reverb_level(&self) -> f32 {
        self.reverb
    }

    fn delay_level(&self) -> f32 {
        self.delay
    }
}

/// a low-frequency sawtooth synth with envelope and lpf18 filter
pub struct LFSawSynth<const BUFSIZE: usize, const NCHAN: usize> {
    oscillator: LFSaw<BUFSIZE>,
    filter: Lpf18<BUFSIZE>,
    envelope: ASREnvelope<BUFSIZE>,
    balance: PanChan<BUFSIZE, NCHAN>,
    reverb: f32,
    delay: f32,
}

impl<const BUFSIZE: usize, const NCHAN: usize> LFSawSynth<BUFSIZE, NCHAN> {
    pub fn new(sr: f32) -> Self {
        LFSawSynth {
            oscillator: LFSaw::new(100.0, 0.8, sr),
            filter: Lpf18::new(1500.0, 0.5, 0.1, sr),
            envelope: ASREnvelope::new(sr, 1.0, 0.002, 0.02, 0.08),
            balance: PanChan::new(),
            reverb: 0.0,
            delay: 0.0,
        }
    }
}

impl<const BUFSIZE: usize, const NCHAN: usize> Synth<BUFSIZE, NCHAN>
    for LFSawSynth<BUFSIZE, NCHAN>
{
    fn set_parameter(&mut self, par: SynthParameter, val: f32) {
        self.oscillator.set_parameter(par, val);
        self.filter.set_parameter(par, val);
        self.envelope.set_parameter(par, val);
        self.balance.set_parameter(par, val);

        match par {
            SynthParameter::ReverbMix => self.reverb = val,
            SynthParameter::DelayMix => self.delay = val,
            _ => (),
        };
    }

    fn finish(&mut self) {
        self.envelope.finish();
    }

    fn is_finished(&self) -> bool {
        self.envelope.is_finished()
    }

    fn get_next_block(&mut self, start_sample: usize, sample_buffers: &Vec<Vec<f32>>) -> [[f32; BUFSIZE]; NCHAN] {
        let mut out: [f32; BUFSIZE] = self.oscillator.get_next_block(start_sample, sample_buffers);
        out = self.filter.process_block(out, start_sample);
        out = self.envelope.process_block(out, start_sample);
        self.balance.process_block(out)
    }

    fn reverb_level(&self) -> f32 {
        self.reverb
    }

    fn delay_level(&self) -> f32 {
        self.delay
    }
}

/// a low-frequency (non-bandlimited) squarewave synth with envelope and lpf18 filter
pub struct LFSquareSynth<const BUFSIZE: usize, const NCHAN: usize> {
    oscillator: LFSquare<BUFSIZE>,
    filter: Lpf18<BUFSIZE>,
    envelope: ASREnvelope<BUFSIZE>,
    balance: PanChan<BUFSIZE, NCHAN>,
    reverb: f32,
    delay: f32,
}

impl<const BUFSIZE: usize, const NCHAN: usize> LFSquareSynth<BUFSIZE, NCHAN> {
    pub fn new(sr: f32) -> Self {
        LFSquareSynth {
            oscillator: LFSquare::new(100.0, 0.4, 0.8, sr),
            filter: Lpf18::new(1500.0, 0.5, 0.1, sr),
            envelope: ASREnvelope::new(sr, 1.0, 0.002, 0.02, 0.08),
            balance: PanChan::new(),
            reverb: 0.0,
            delay: 0.0,
        }
    }
}

impl<const BUFSIZE: usize, const NCHAN: usize> Synth<BUFSIZE, NCHAN>
    for LFSquareSynth<BUFSIZE, NCHAN>
{
    fn set_parameter(&mut self, par: SynthParameter, val: f32) {
        self.oscillator.set_parameter(par, val);
        self.filter.set_parameter(par, val);
        self.envelope.set_parameter(par, val);
        self.balance.set_parameter(par, val);

        match par {
            SynthParameter::ReverbMix => self.reverb = val,
            SynthParameter::DelayMix => self.delay = val,
            _ => (),
        };
    }

    fn finish(&mut self) {
        self.envelope.finish();
    }

    fn is_finished(&self) -> bool {
        self.envelope.is_finished()
    }

    fn get_next_block(&mut self, start_sample: usize, sample_buffers: &Vec<Vec<f32>>) -> [[f32; BUFSIZE]; NCHAN] {
        let mut out: [f32; BUFSIZE] = self.oscillator.get_next_block(start_sample, sample_buffers);
        out = self.filter.process_block(out, start_sample);
        out = self.envelope.process_block(out, start_sample);
        self.balance.process_block(out)
    }

    fn reverb_level(&self) -> f32 {
        self.reverb
    }

    fn delay_level(&self) -> f32 {
        self.delay
    }
}

/// a sampler with envelope etc.
pub struct NChannelSampler<const BUFSIZE: usize, const NCHAN: usize> {
    sampler: Sampler<BUFSIZE>,
    envelope: ASREnvelope<BUFSIZE>,
    hpf: BiquadHpf<BUFSIZE>,
    peak_eq: PeakEq<BUFSIZE>,
    lpf: Lpf18<BUFSIZE>,
    balance: PanChan<BUFSIZE, NCHAN>,
    reverb: f32,
    delay: f32,
}

impl<const BUFSIZE: usize, const NCHAN: usize> NChannelSampler<BUFSIZE, NCHAN> {
    pub fn with_bufnum_len(bufnum: usize, buflen: usize, sr: f32) -> NChannelSampler<BUFSIZE, NCHAN> {
        let dur = (buflen as f32 / sr) - 0.0002;

        NChannelSampler {
            sampler: Sampler::with_bufnum_len(bufnum, buflen, true),
            envelope: ASREnvelope::new(sr, 1.0, 0.0001, dur, 0.0001),
            hpf: BiquadHpf::new(20.0, 0.3, sr),
            peak_eq: PeakEq::new(700.0, 100.0, 0.0, sr),
            lpf: Lpf18::new(19500.0, 0.01, 0.01, sr),
            balance: PanChan::new(),
            reverb: 0.0,
            delay: 0.0,
        }
    }
}

impl<const BUFSIZE: usize, const NCHAN: usize> Synth<BUFSIZE, NCHAN>
    for NChannelSampler<BUFSIZE, NCHAN>
{
    fn set_parameter(&mut self, par: SynthParameter, val: f32) {
        self.sampler.set_parameter(par, val);
        self.hpf.set_parameter(par, val);
        self.peak_eq.set_parameter(par, val);
        self.lpf.set_parameter(par, val);
        self.envelope.set_parameter(par, val);
        self.balance.set_parameter(par, val);

        match par {
            SynthParameter::ReverbMix => self.reverb = val,
            SynthParameter::DelayMix => self.delay = val,
            _ => (),
        };
    }

    fn finish(&mut self) {
        self.envelope.finish();
    }

    fn is_finished(&self) -> bool {
        self.envelope.is_finished()
    }

    fn get_next_block(&mut self, start_sample: usize, sample_buffers: &Vec<Vec<f32>>) -> [[f32; BUFSIZE]; NCHAN] {
        let mut out: [f32; BUFSIZE] = self.sampler.get_next_block(start_sample, sample_buffers);
        out = self.hpf.process_block(out, start_sample);
        out = self.peak_eq.process_block(out, start_sample);
        out = self.lpf.process_block(out, start_sample);
        out = self.envelope.process_block(out, start_sample);
        self.balance.process_block(out)
    }

    fn reverb_level(&self) -> f32 {
        self.reverb
    }

    fn delay_level(&self) -> f32 {
        self.delay
    }
}
