pub mod ambisonics;
pub mod convolution_reverb;
pub mod convolver;
pub mod delay;
pub mod envelopes;
pub mod filters;
pub mod freeverb;
pub mod interpolation;
pub mod mod_env;
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
    Attack,                   // 0
    Decay,                    // 1
    DelayDampeningFrequency,  // 2
    DelayFeedback,            // 3
    DelayMix,                 // 4
    DelayTime,                // 5
    DelayRate,                // 6
    Duration,                 // 7
    PitchFrequency,           // 8
    PitchNote,                // 9
    HighpassCutoffFrequency,  // 10
    HighpassQFactor,          // 11
    EnvelopeLevel,            // 12
    OscillatorAmplitude,      // 13 (oscillator amplitude)
    OscillatorPhaseRelative,  // 14 (radians)
    OscillatorPhaseEffective, // 15 (the target value or effective offset dependent on the amplitude)
    LowpassCutoffFrequency,   // 16
    LowpassQFactor,           // 17
    LowpassFilterDistortion,  // 18
    PeakFrequency,            // 19
    PeakGain,                 // 20
    PeakQFactor,              // 21
    Pulsewidth,               // 22
    PlaybackRate,             // 23
    PlaybackStart,            // 24
    PlaybackLoop,             // 25
    Release,                  // 26
    ReverbDampening,          // 27
    ReverbMix,                // 28
    ReverbRoomsize,           // 29
    SampleBufferNumber,       // 30
    Samplerate,               // 31
    ChannelPosition,          // 32
    AmbisonicAzimuth,         // 33
    AmbisonicElevation,       // 34
    Sustain,                  // 35
    Wavetable,                // 36
    Wavematrix,               // 37
    WavematrixTableIndex,     // 38
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
    Lfo(f32, f32, f32, f32, f32, ValOp), // sine lfo - init val, freq, phase, amp, add, operation (mul, add, sub, div, replace)
    LFSaw(f32, f32, f32, f32, f32, ValOp), // saw lfo - init val, freq, phase, amp, add, operation (mul, add, sub, div, replace)
    LFRSaw(f32, f32, f32, f32, f32, ValOp), // rev saw lfo - init val, freq, phase, amp, add, operation (mul, add, sub, div, replace)
    LFSquare(f32, f32, f32, f32, f32, ValOp), // square lfo - init val, freq, pw, amp, add, operation (mul, add, sub, div, replace)
    LFTri(f32, f32, f32, f32, f32, ValOp), // tri lfo - init val, freq, phase amp, add, operation (mul, add, sub, div, replace)
    LinRamp(f32, f32, f32, ValOp),         // linear ramp - from, to, time
    LogRamp(f32, f32, f32, ValOp),         // logarithmic ramp - from, to, time
    ExpRamp(f32, f32, f32, ValOp),         // exponential ramp - from, to, time
    MultiPointEnvelope(Vec<mod_env::SegmentInfo>, bool, ValOp), // segments, loop ...
}

/// oscillators, the sampler, etc are sources
pub trait MonoSource<const BUFSIZE: usize>: MonoSourceClone<BUFSIZE> {
    fn set_parameter(&mut self, par: SynthParameterLabel, value: &SynthParameterValue);
    fn set_modulator(&mut self, par: SynthParameterLabel, init: f32, modulator: Modulator<BUFSIZE>);
    fn finish(&mut self);
    fn reset(&mut self);
    fn is_finished(&self) -> bool;
    fn get_next_block(&mut self, start_sample: usize, in_buffers: &[Vec<f32>]) -> [f32; BUFSIZE];
}

pub trait MonoSourceClone<const BUFSIZE: usize> {
    fn clone_box(&self) -> Box<dyn MonoSource<BUFSIZE> + Send + Sync>;
}

impl<const BUFSIZE: usize, T> MonoSourceClone<BUFSIZE> for T
where
    T: 'static + MonoSource<BUFSIZE> + Clone + Send + Sync,
{
    fn clone_box(&self) -> Box<dyn MonoSource<BUFSIZE> + Send + Sync> {
        Box::new(self.clone())
    }
}

impl<const BUFSIZE: usize> Clone for Box<dyn MonoSource<BUFSIZE> + Send + Sync> {
    fn clone(&self) -> Box<dyn MonoSource<BUFSIZE> + Send + Sync> {
        self.clone_box()
    }
}

/// filters etc are effects
pub trait MonoEffect<const BUFSIZE: usize> {
    fn finish(&mut self);
    fn is_finished(&self) -> bool;
    fn set_parameter(&mut self, par: SynthParameterLabel, value: &SynthParameterValue);
    fn set_modulator(&mut self, par: SynthParameterLabel, init: f32, modulator: Modulator<BUFSIZE>);
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
    fn set_modulator(&mut self, par: SynthParameterLabel, init: f32, modulator: Modulator<BUFSIZE>);
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
