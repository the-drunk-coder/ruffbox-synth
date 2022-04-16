pub mod ambisonics;
pub mod convolution_reverb;
pub mod convolver;
pub mod delay;
pub mod envelopes;
pub mod filters;
pub mod freeverb;
pub mod oscillators;
pub mod routing;
pub mod sampler;
pub mod synths;

#[derive(Clone, Copy)]
pub enum SynthState {
    Fresh, // Fresh Synths for everyone !!!
    Finished,
}

/// a collection of common parameters
#[allow(dead_code)]
#[repr(C)]
#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub enum SynthParameterLabel {
    Attack,                  // 0
    Decay,                   // 1
    DelayDampeningFrequency, // 2
    DelayFeedback,           // 3
    DelayMix,                // 4
    DelayTime,               // 5
    Duration,                // 6
    PitchFrequency,          // 7
    PitchNote,               // 8
    HighpassCutoffFrequency, // 9
    HighpassQFactor,         // 10
    Level,                   // 11
    LowpassCutoffFrequency,  // 12
    LowpassQFactor,          // 13
    LowpassFilterDistortion, // 14
    PeakFrequency,           // 15
    PeakGain,                // 16
    PeakQFactor,             // 17
    Pulsewidth,              // 18
    PlaybackRate,            // 19
    PlaybackStart,           // 20
    PlaybackLoop,            // 21
    Release,                 // 22
    ReverbDampening,         // 23
    ReverbMix,               // 24
    ReverbRoomsize,          // 25
    SampleBufferNumber,      // 26
    Samplerate,              // 27
    ChannelPosition,         // 28
    AmbisonicAzimuth,        // 29
    AmbisonicElevation,      // 30
    Sustain,                 // 31
    Wavetable,               // 32
}

#[derive(Clone)]
pub enum SynthParameterValue {
    ScalarF32(f32),
    ScalarU32(u32),
    ScalarUsize(usize),
    VecF32(Vec<f32>),
}

#[repr(C)]
pub enum SourceType {
    Sampler,
    LiveSampler,
    FrozenSampler,
    SineOsc,
    SineSynth,
    LFCubSynth,
    LFSawSynth,
    LFSquareSynth,
    LFTriangleSynth,
    RissetBell,
    Wavetable,
}

#[derive(Clone)]
pub enum ModulatorOperation {
    Replace,
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Clone)]
pub struct Modulator<const BUFSIZE: usize> {
    pub param: SynthParameterLabel,
    pub op: ModulatorOperation,
    pub inlet_block: [f32; BUFSIZE],
}

pub trait MonoSource<const BUFSIZE: usize> {
    fn set_parameter(&mut self, par: SynthParameterLabel, value: &SynthParameterValue);
    fn finish(&mut self);
    fn is_finished(&self) -> bool;
    fn get_next_block(
        &mut self,
        start_sample: usize,
        in_buffers: &[Vec<f32>],
        inlets: &[Modulator<BUFSIZE>],
    ) -> [f32; BUFSIZE];
}

pub trait MonoEffect<const BUFSIZE: usize> {
    fn finish(&mut self);
    fn is_finished(&self) -> bool;
    fn set_parameter(&mut self, par: SynthParameterLabel, value: &SynthParameterValue);
    fn process_block(&mut self, block: [f32; BUFSIZE], start_sample: usize) -> [f32; BUFSIZE];
}

pub trait MultichannelReverb<const BUFSIZE: usize, const NCHAN: usize> {
    fn set_parameter(&mut self, par: SynthParameterLabel, value: &SynthParameterValue);
    fn process(&mut self, block: [[f32; BUFSIZE]; NCHAN]) -> [[f32; BUFSIZE]; NCHAN];
}

pub trait Synth<const BUFSIZE: usize, const NCHAN: usize> {
    fn set_parameter(&mut self, par: SynthParameterLabel, value: &SynthParameterValue);
    fn finish(&mut self);
    fn is_finished(&self) -> bool;
    fn get_next_block(
        &mut self,
        start_sample: usize,
        in_buffers: &[Vec<f32>],
    ) -> [[f32; BUFSIZE]; NCHAN];
    fn reverb_level(&self) -> f32;
    fn delay_level(&self) -> f32;
}
