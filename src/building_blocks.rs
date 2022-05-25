pub mod ambisonics;
pub mod convolution_reverb;
pub mod convolver;
pub mod delay;
pub mod envelopes;
pub mod filters;
pub mod freeverb;
pub mod interpolation;
pub mod modulator;
pub mod oscillators;
pub mod routing;
pub mod sampler;

pub use crate::building_blocks::modulator::Modulator;

#[derive(Clone, Copy)]
pub enum SynthState {
    Fresh, // Fresh Synths for everyone !!!
    Finished,
}

/// a collection of common parameters
#[allow(dead_code)]
#[repr(C)]
#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub enum SynthParameterLabel {
    Attack,                  // 0
    Decay,                   // 1
    DelayDampeningFrequency, // 2
    DelayFeedback,           // 3
    DelayMix,                // 4
    DelayTime,               // 5
    DelayRate,               // 6
    Duration,                // 7
    PitchFrequency,          // 8
    PitchNote,               // 9
    HighpassCutoffFrequency, // 10
    HighpassQFactor,         // 11
    EnvelopeLevel,           // 12
    OscillatorLevel,         // 13
    LowpassCutoffFrequency,  // 14
    LowpassQFactor,          // 15
    LowpassFilterDistortion, // 16
    PeakFrequency,           // 17
    PeakGain,                // 18
    PeakQFactor,             // 19
    Pulsewidth,              // 20
    PlaybackRate,            // 21
    PlaybackStart,           // 22
    PlaybackLoop,            // 23
    Release,                 // 24
    ReverbDampening,         // 25
    ReverbMix,               // 26
    ReverbRoomsize,          // 27
    SampleBufferNumber,      // 28
    Samplerate,              // 29
    ChannelPosition,         // 30
    AmbisonicAzimuth,        // 31
    AmbisonicElevation,      // 32
    Sustain,                 // 33
    Wavetable,               // 34
    Wavematrix,              // 35
    WavematrixTableIndex,    // 36
}

#[derive(Clone, Copy)]
pub enum ValOp {
    Replace,
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Clone)]
pub enum SynthParameterValue {
    ScalarF32(f32),
    ScalarU32(u32),
    ScalarUsize(usize),
    VecF32(Vec<f32>),
    MatrixF32((usize, usize), Vec<Vec<f32>>), // dimension, content
    Lfo(f32, f32, f32, ValOp), // sine lfo - init val, freq, range, operation (mul, add, sub, div, replace)
}

/// oscillators, the sampler, etc are sources
pub trait MonoSource<const BUFSIZE: usize> {
    fn set_parameter(&mut self, par: SynthParameterLabel, value: &SynthParameterValue);
    fn finish(&mut self);
    fn is_finished(&self) -> bool;
    fn get_next_block(&mut self, start_sample: usize, in_buffers: &[Vec<f32>]) -> [f32; BUFSIZE];
}

/// filters etc are effects
pub trait MonoEffect<const BUFSIZE: usize> {
    fn finish(&mut self);
    fn is_finished(&self) -> bool;
    fn set_parameter(&mut self, par: SynthParameterLabel, value: &SynthParameterValue);
    fn process_block(
        &mut self,
        block: [f32; BUFSIZE],
        start_sample: usize,
        in_buffers: &[Vec<f32>],
    ) -> [f32; BUFSIZE];
}

/// there's a freeverb- and a convolution-based implementation
pub trait MultichannelReverb<const BUFSIZE: usize, const NCHAN: usize> {
    fn set_parameter(&mut self, par: SynthParameterLabel, value: &SynthParameterValue);
    fn process(&mut self, block: [[f32; BUFSIZE]; NCHAN]) -> [[f32; BUFSIZE]; NCHAN];
}

/// This is where the building blocks come together
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
