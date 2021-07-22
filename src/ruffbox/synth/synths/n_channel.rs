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

    fn get_next_block(
        &mut self,
        start_sample: usize,
        sample_buffers: &Vec<Vec<f32>>,
    ) -> [[f32; BUFSIZE]; NCHAN] {
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

    fn get_next_block(
        &mut self,
        start_sample: usize,
        sample_buffers: &Vec<Vec<f32>>,
    ) -> [[f32; BUFSIZE]; NCHAN] {
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

/// a triangle synth with envelope etc.
pub struct LFTriSynth<const BUFSIZE: usize, const NCHAN: usize> {
    oscillator: LFTri<BUFSIZE>,
    envelope: ASREnvelope<BUFSIZE>,
    balance: PanChan<BUFSIZE, NCHAN>,
    reverb: f32,
    delay: f32,
}

impl<const BUFSIZE: usize, const NCHAN: usize> LFTriSynth<BUFSIZE, NCHAN> {
    pub fn new(sr: f32) -> Self {
        LFTriSynth {
            oscillator: LFTri::new(440.0, 0.5, sr),
            envelope: ASREnvelope::new(sr, 0.3, 0.05, 0.1, 0.05),
            balance: PanChan::new(),
            reverb: 0.0,
            delay: 0.0,
        }
    }
}

impl<const BUFSIZE: usize, const NCHAN: usize> Synth<BUFSIZE, NCHAN> for LFTriSynth<BUFSIZE, NCHAN> {
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

    fn get_next_block(
        &mut self,
        start_sample: usize,
        sample_buffers: &Vec<Vec<f32>>,
    ) -> [[f32; BUFSIZE]; NCHAN] {
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

    fn get_next_block(
        &mut self,
        start_sample: usize,
        sample_buffers: &Vec<Vec<f32>>,
    ) -> [[f32; BUFSIZE]; NCHAN] {
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

    fn get_next_block(
        &mut self,
        start_sample: usize,
        sample_buffers: &Vec<Vec<f32>>,
    ) -> [[f32; BUFSIZE]; NCHAN] {
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
    pub fn with_bufnum_len(
        bufnum: usize,
        buflen: usize,
        sr: f32,
    ) -> NChannelSampler<BUFSIZE, NCHAN> {
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

    fn get_next_block(
        &mut self,
        start_sample: usize,
        sample_buffers: &Vec<Vec<f32>>,
    ) -> [[f32; BUFSIZE]; NCHAN] {
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


/// 11-partial risset bell, modeled after Frederik Oloffson's Supercollider port
pub struct RissetBell<const BUFSIZE: usize, const NCHAN: usize> {
    oscillators: [SineOsc<BUFSIZE>; 11],
    envelopes: [ExpPercEnvelope<BUFSIZE>; 11],
    main_envelope: ASREnvelope<BUFSIZE>,
    amps: [f32; 11],
    durs: [f32; 11],
    freqs: [f32; 11],
    dets: [f32; 11],
    lpf: Lpf18<BUFSIZE>,
    balance: PanChan<BUFSIZE, NCHAN>,
    atk: f32,
    sus: f32,
    rel: f32,
    freq: f32,
    main_level: f32,
    reverb: f32,
    delay: f32,
}

impl<const BUFSIZE: usize, const NCHAN: usize> RissetBell<BUFSIZE, NCHAN> {
    pub fn new(sr: f32) -> RissetBell<BUFSIZE, NCHAN> {        

	let mut bell = RissetBell {
            oscillators: [SineOsc::new(440.0, 1.0, sr); 11],
            envelopes: [ExpPercEnvelope::new(sr, 1.0, 0.005, 0.0, 0.05); 11],
	    main_envelope: ASREnvelope::new(sr, 1.0, 0.05, 0.5, 0.05),
	    amps: [1.0, 0.67, 1.0, 1.8, 2.67, 1.67, 1.46, 1.33, 1.33, 1.0, 1.33],
	    durs: [1.0, 0.9, 0.65, 0.55, 0.325, 0.35, 0.25, 0.2, 0.15, 0.1, 0.075],
	    freqs: [0.56, 0.56, 0.92, 0.92, 1.19, 1.7, 2.0, 2.74, 3.0, 3.76, 4.07],
	    dets: [0.0, 1.0, 0.0, 1.7, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            lpf: Lpf18::new(19500.0, 0.01, 0.01, sr),
            balance: PanChan::new(),
	    atk: 0.05,
	    sus: 0.7,
	    rel: 0.05,
	    main_level: 0.7,
	    freq: 1000.0,
            reverb: 0.0,
            delay: 0.0,
        };

	// init with some default frequency
	let freq = 1000.0;
	let length = 0.8;
	for i in 0..11 {
	    // set envelope params
	    bell.envelopes[i].set_parameter(SynthParameter::Level, bell.amps[i] * bell.main_level);
	    bell.envelopes[i].set_parameter(SynthParameter::Release, bell.durs[i] * length);

	    // set oscillator params
	    bell.oscillators[i].set_parameter(SynthParameter::PitchFrequency, freq * bell.freqs[i] + bell.dets[i]);			    
	}
	
	bell
    }
}

impl<const BUFSIZE: usize, const NCHAN: usize> Synth<BUFSIZE, NCHAN>
    for RissetBell<BUFSIZE, NCHAN>
{
    fn set_parameter(&mut self, par: SynthParameter, val: f32) {

        self.lpf.set_parameter(par, val);
	self.main_envelope.set_parameter(par, val);        
        self.balance.set_parameter(par, val);

	let mut update_internals = false;
        match par {
            SynthParameter::ReverbMix => self.reverb = val,
            SynthParameter::DelayMix => self.delay = val,
	    SynthParameter::PitchFrequency => {
		self.freq = val;
		update_internals = true;
	    },
	    SynthParameter::Attack => {
		self.atk = val;
		update_internals = true;
	    },
	    SynthParameter::Sustain => {
		self.sus = val;
		update_internals = true;
	    },
	    SynthParameter::Release => {
		self.rel = val;
		update_internals = true;
	    },
	    SynthParameter::Level => {
		self.main_level = val;
		update_internals = true;
	    },	
            _ => (),
        };

	if update_internals {
	    let length = self.atk + self.sus + self.rel;
	    for i in 0..11 {
		// set envelope params
		self.envelopes[i].set_parameter(SynthParameter::Level, self.amps[i] * self.main_level);
		self.envelopes[i].set_parameter(SynthParameter::Release, self.durs[i] * length);
		
		// set oscillator params
		self.oscillators[i].set_parameter(SynthParameter::PitchFrequency, self.freq * self.freqs[i] + self.dets[i]);			    
	    }
	}
    }

    fn finish(&mut self) {
        self.main_envelope.finish();
    }

    fn is_finished(&self) -> bool {
        self.main_envelope.is_finished()
    }

    fn get_next_block(
        &mut self,
        start_sample: usize,
        sample_buffers: &Vec<Vec<f32>>,
    ) -> [[f32; BUFSIZE]; NCHAN] {
	// first osc
	let mut out: [f32; BUFSIZE] = self.oscillators[0].get_next_block(start_sample, sample_buffers);
	out = self.envelopes[0].process_block(out, start_sample);

	// rest	
	for i in 1..11 {
	    let mut tmp_out: [f32; BUFSIZE] = self.oscillators[i].get_next_block(start_sample, sample_buffers);
	    tmp_out = self.envelopes[i].process_block(tmp_out, start_sample);

	    for s in 0..BUFSIZE {
		out[s] += tmp_out[s];
	    }	    
	}

	out = self.lpf.process_block(out, start_sample);
	out = self.main_envelope.process_block(out, start_sample);
        self.balance.process_block(out)
    }

    fn reverb_level(&self) -> f32 {
        self.reverb
    }

    fn delay_level(&self) -> f32 {
        self.delay
    }
}
