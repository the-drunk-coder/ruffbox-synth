pub mod ambisonics;
pub mod convolver;
pub mod delay;
pub mod envelopes;
pub mod filters;
pub mod interpolation;

pub mod modulator;
pub mod oscillators;
pub mod reverb;
pub mod routing;
pub mod sampler;
pub mod waveshaper;

pub use crate::building_blocks::envelopes::source_env::*;
pub use crate::building_blocks::modulator::Modulator;

/// currently available oscillator types
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub enum OscillatorType {
    Sine,
    LFTri,
    LFSquare,
    LFSaw,
    LFRsaw,
    LFCub,
    FMSquare,
    FMSaw,
    FMTri,
    WTSaw,
    Wavetable,
    Wavematrix,
    WhiteNoise,
    BrownNoise,
}

/// the available filter types.
/// dummy filter just passes the block through unprocessed.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub enum FilterType {
    Dummy,
    Lpf18,
    BiquadHpf12dB,
    BiquadLpf12dB,
    BiquadHpf24dB,
    BiquadLpf24dB,
    ButterworthLpf(usize),
    ButterworthHpf(usize),
    PeakEQ,
}

/// used to determine whether something has finished
/// especially envelopes (oscillators never finish)
#[derive(Clone, Copy)]
pub enum SynthState {
    Fresh, // Fresh Synths for everyone !!!
    Finished,
}

/// a collection of common parameters that should be enough to
/// control just about anything
#[allow(dead_code)]
#[repr(C)]
#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub enum SynthParameterLabel {
    Attack,                   // 0
    AttackType,               // 1
    AttackPeakLevel,          // 2
    Decay,                    // 3
    DecayType,                // 4
    DelayDampeningFrequency,  // 5
    DelayFeedback,            // 6
    DelayMix,                 // 7
    DelayTime,                // 8
    DelayRate,                // 9
    Duration,                 // 10
    Envelope,                 // 11
    PitchFrequency,           // 12
    PitchNote,                // 13
    HighpassCutoffFrequency,  // 14
    HighpassQFactor,          // 15
    HighpassFilterType,       // 16
    EnvelopeLevel,            // 17
    OscillatorAmplitude,      // 18 (oscillator amplitude)
    OscillatorPhaseRelative,  // 19 (radians)
    OscillatorPhaseEffective, // 20 (the target value or effective offset dependent on the amplitude)
    LowpassCutoffFrequency,   // 21
    LowpassQFactor,           // 22
    LowpassFilterDistortion,  // 23
    LowpassFilterType,        // 24
    PeakFrequency,            // 25
    PeakGain,                 // 26
    PeakBandwidth,            // 27
    Pulsewidth,               // 28
    PlaybackRate,             // 29
    PlaybackStart,            // 30
    PlaybackLoop,             // 31
    Release,                  // 32
    ReleaseType,              // 33
    ReverbDampening,          // 34
    ReverbMix,                // 35
    ReverbRoomsize,           // 36
    SampleBufferNumber,       // 37
    Samplerate,               // 38
    ChannelPosition,          // 39
    AmbisonicAzimuth,         // 40
    AmbisonicElevation,       // 41
    Sustain,                  // 42
    Wavetable,                // 43
    Wavematrix,               // 44
    WavematrixTableIndex,     // 45
    WaveshaperMix,            // 46
}

/// the value operation is defined on parameters
#[derive(Clone, Copy, Debug)]
pub enum ValOp {
    Replace,
    Add,
    Subtract,
    Multiply,
    Divide,
}

/// in an envelope, each segment can have a certain curve shape
#[derive(Clone, Copy, Debug)]
pub enum EnvelopeSegmentType {
    Lin,
    Log,
    Exp,
    Sin,
    Cos,
    Constant,
}

pub enum SampleBuffer {
    Mono(Vec<f32>),
    Stereo(Vec<f32>, Vec<f32>),
    Placeholder,
}

/// defines an envelope segment
#[derive(Clone, Copy, Debug)]
pub struct EnvelopeSegmentInfo {
    pub from: f32, // level
    pub to: f32,   // level
    pub time: f32, // transition time
    pub segment_type: EnvelopeSegmentType,
}

// from an outside perspective, there can be modulator-valued parameters (like, an lfo-valued parameter)
#[derive(Clone, Debug)]
#[rustfmt::skip]
pub enum SynthParameterValue {    
    ScalarF32(f32),
    ScalarU32(u32),
    ScalarUsize(usize),
    VecF32(Vec<f32>),
    FilterType(FilterType), // these aren't really treated as parameters so far, but as a pragmatic solution that's ok for now ...    
    MatrixF32((usize, usize), Vec<Vec<f32>>), // dimension, content
    // lfo param order - init val, freq, phase, amp, add, operation (mul, add, sub, div, replace)
    Lfo(f32, Box<SynthParameterValue>, f32, Box<SynthParameterValue>, f32, ValOp), // sine lfo
    LFSaw(f32, Box<SynthParameterValue>, f32, Box<SynthParameterValue>, f32, ValOp), // sawtooth lfo
    LFRSaw(f32, Box<SynthParameterValue>, f32, Box<SynthParameterValue>, f32, ValOp), // reverse sawtooth lfo
    LFTri(f32, Box<SynthParameterValue>, f32, Box<SynthParameterValue>, f32, ValOp), // triangle wave lfo
    LFSquare(f32, Box<SynthParameterValue>, f32, Box<SynthParameterValue>, f32, ValOp), // squarewave lfo
    LinRamp(f32, f32, f32, ValOp), // linear ramp - from, to, time
    LogRamp(f32, f32, f32, ValOp), // logarithmic ramp - from, to, time
    ExpRamp(f32, f32, f32, ValOp), // exponential ramp - from, to, time,
    EnvelopeSegmentType(EnvelopeSegmentType),
    MultiPointEnvelope(Vec<EnvelopeSegmentInfo>, bool, ValOp), // segments, loop ...
}

// but in practice, it's not that easy ...
// so we need some helper enums
#[derive(Clone)]
pub enum ValueOrModulator<const BUFSIZE: usize> {
    Val(SynthParameterValue),
    Mod(f32, Modulator<BUFSIZE>),
}

pub fn resolve_parameter_value<const BUFSIZE: usize>(
    par: SynthParameterLabel,
    val: &SynthParameterValue,
    samplerate: f32,
) -> ValueOrModulator<BUFSIZE> {
    match val {
        SynthParameterValue::Lfo(init, freq, eff_phase, amp, add, op) => ValueOrModulator::Mod(
            *init,
            match par {
                SynthParameterLabel::LowpassCutoffFrequency => Modulator::lfo(
                    *op,
                    resolve_parameter_value(SynthParameterLabel::PitchFrequency, freq, samplerate),
                    *eff_phase,
                    resolve_parameter_value(
                        SynthParameterLabel::OscillatorAmplitude,
                        amp,
                        samplerate,
                    ),
                    *add,
                    true,
                    false,
                    samplerate,
                ),
                SynthParameterLabel::HighpassCutoffFrequency => Modulator::lfo(
                    *op,
                    resolve_parameter_value(SynthParameterLabel::PitchFrequency, freq, samplerate),
                    *eff_phase,
                    resolve_parameter_value(
                        SynthParameterLabel::OscillatorAmplitude,
                        amp,
                        samplerate,
                    ),
                    *add,
                    true,
                    false,
                    samplerate,
                ),
                SynthParameterLabel::PeakFrequency => Modulator::lfo(
                    *op,
                    resolve_parameter_value(SynthParameterLabel::PitchFrequency, freq, samplerate),
                    *eff_phase,
                    resolve_parameter_value(
                        SynthParameterLabel::OscillatorAmplitude,
                        amp,
                        samplerate,
                    ),
                    *add,
                    true,
                    false,
                    samplerate,
                ),
                _ => Modulator::lfo(
                    *op,
                    resolve_parameter_value(SynthParameterLabel::PitchFrequency, freq, samplerate),
                    *eff_phase,
                    resolve_parameter_value(
                        SynthParameterLabel::OscillatorAmplitude,
                        amp,
                        samplerate,
                    ),
                    *add,
                    false,
                    false,
                    samplerate,
                ),
            },
        ),
        SynthParameterValue::LFSaw(init, freq, eff_phase, amp, add, op) => ValueOrModulator::Mod(
            *init,
            match par {
                SynthParameterLabel::LowpassCutoffFrequency => Modulator::lfsaw(
                    *op,
                    resolve_parameter_value(SynthParameterLabel::PitchFrequency, freq, samplerate),
                    *eff_phase,
                    resolve_parameter_value(
                        SynthParameterLabel::OscillatorAmplitude,
                        amp,
                        samplerate,
                    ),
                    *add,
                    true,
                    false,
                    samplerate,
                ),
                SynthParameterLabel::HighpassCutoffFrequency => Modulator::lfsaw(
                    *op,
                    resolve_parameter_value(SynthParameterLabel::PitchFrequency, freq, samplerate),
                    *eff_phase,
                    resolve_parameter_value(
                        SynthParameterLabel::OscillatorAmplitude,
                        amp,
                        samplerate,
                    ),
                    *add,
                    true,
                    false,
                    samplerate,
                ),
                SynthParameterLabel::PeakFrequency => Modulator::lfsaw(
                    *op,
                    resolve_parameter_value(SynthParameterLabel::PitchFrequency, freq, samplerate),
                    *eff_phase,
                    resolve_parameter_value(
                        SynthParameterLabel::OscillatorAmplitude,
                        amp,
                        samplerate,
                    ),
                    *add,
                    true,
                    false,
                    samplerate,
                ),
                _ => Modulator::lfsaw(
                    *op,
                    resolve_parameter_value(SynthParameterLabel::PitchFrequency, freq, samplerate),
                    *eff_phase,
                    resolve_parameter_value(
                        SynthParameterLabel::OscillatorAmplitude,
                        amp,
                        samplerate,
                    ),
                    *add,
                    false,
                    false,
                    samplerate,
                ),
            },
        ),
        SynthParameterValue::LFRSaw(init, freq, eff_phase, amp, add, op) => ValueOrModulator::Mod(
            *init,
            match par {
                SynthParameterLabel::LowpassCutoffFrequency => Modulator::lfrsaw(
                    *op,
                    resolve_parameter_value(SynthParameterLabel::PitchFrequency, freq, samplerate),
                    *eff_phase,
                    resolve_parameter_value(
                        SynthParameterLabel::OscillatorAmplitude,
                        amp,
                        samplerate,
                    ),
                    *add,
                    true,
                    false,
                    samplerate,
                ),
                SynthParameterLabel::HighpassCutoffFrequency => Modulator::lfrsaw(
                    *op,
                    resolve_parameter_value(SynthParameterLabel::PitchFrequency, freq, samplerate),
                    *eff_phase,
                    resolve_parameter_value(
                        SynthParameterLabel::OscillatorAmplitude,
                        amp,
                        samplerate,
                    ),
                    *add,
                    true,
                    false,
                    samplerate,
                ),
                SynthParameterLabel::PeakFrequency => Modulator::lfrsaw(
                    *op,
                    resolve_parameter_value(SynthParameterLabel::PitchFrequency, freq, samplerate),
                    *eff_phase,
                    resolve_parameter_value(
                        SynthParameterLabel::OscillatorAmplitude,
                        amp,
                        samplerate,
                    ),
                    *add,
                    true,
                    false,
                    samplerate,
                ),
                _ => Modulator::lfrsaw(
                    *op,
                    resolve_parameter_value(SynthParameterLabel::PitchFrequency, freq, samplerate),
                    *eff_phase,
                    resolve_parameter_value(
                        SynthParameterLabel::OscillatorAmplitude,
                        amp,
                        samplerate,
                    ),
                    *add,
                    false,
                    false,
                    samplerate,
                ),
            },
        ),
        SynthParameterValue::LFTri(init, freq, eff_phase, amp, add, op) => ValueOrModulator::Mod(
            *init,
            match par {
                SynthParameterLabel::LowpassCutoffFrequency => Modulator::lftri(
                    *op,
                    resolve_parameter_value(SynthParameterLabel::PitchFrequency, freq, samplerate),
                    *eff_phase,
                    resolve_parameter_value(
                        SynthParameterLabel::OscillatorAmplitude,
                        amp,
                        samplerate,
                    ),
                    *add,
                    true,
                    false,
                    samplerate,
                ),
                SynthParameterLabel::HighpassCutoffFrequency => Modulator::lftri(
                    *op,
                    resolve_parameter_value(SynthParameterLabel::PitchFrequency, freq, samplerate),
                    *eff_phase,
                    resolve_parameter_value(
                        SynthParameterLabel::OscillatorAmplitude,
                        amp,
                        samplerate,
                    ),
                    *add,
                    true,
                    false,
                    samplerate,
                ),
                SynthParameterLabel::PeakFrequency => Modulator::lftri(
                    *op,
                    resolve_parameter_value(SynthParameterLabel::PitchFrequency, freq, samplerate),
                    *eff_phase,
                    resolve_parameter_value(
                        SynthParameterLabel::OscillatorAmplitude,
                        amp,
                        samplerate,
                    ),
                    *add,
                    true,
                    false,
                    samplerate,
                ),
                _ => Modulator::lftri(
                    *op,
                    resolve_parameter_value(SynthParameterLabel::PitchFrequency, freq, samplerate),
                    *eff_phase,
                    resolve_parameter_value(
                        SynthParameterLabel::OscillatorAmplitude,
                        amp,
                        samplerate,
                    ),
                    *add,
                    false,
                    false,
                    samplerate,
                ),
            },
        ),
        SynthParameterValue::LFSquare(init, freq, pw, amp, add, op) => ValueOrModulator::Mod(
            *init,
            match par {
                SynthParameterLabel::LowpassCutoffFrequency => Modulator::lfsquare(
                    *op,
                    resolve_parameter_value(SynthParameterLabel::PitchFrequency, freq, samplerate),
                    *pw,
                    resolve_parameter_value(
                        SynthParameterLabel::OscillatorAmplitude,
                        amp,
                        samplerate,
                    ),
                    *add,
                    true,
                    false,
                    samplerate,
                ),
                SynthParameterLabel::HighpassCutoffFrequency => Modulator::lfsquare(
                    *op,
                    resolve_parameter_value(SynthParameterLabel::PitchFrequency, freq, samplerate),
                    *pw,
                    resolve_parameter_value(
                        SynthParameterLabel::OscillatorAmplitude,
                        amp,
                        samplerate,
                    ),
                    *add,
                    true,
                    false,
                    samplerate,
                ),
                SynthParameterLabel::PeakFrequency => Modulator::lfsquare(
                    *op,
                    resolve_parameter_value(SynthParameterLabel::PitchFrequency, freq, samplerate),
                    *pw,
                    resolve_parameter_value(
                        SynthParameterLabel::OscillatorAmplitude,
                        amp,
                        samplerate,
                    ),
                    *add,
                    true,
                    false,
                    samplerate,
                ),
                _ => Modulator::lfsquare(
                    *op,
                    resolve_parameter_value(SynthParameterLabel::PitchFrequency, freq, samplerate),
                    *pw,
                    resolve_parameter_value(
                        SynthParameterLabel::OscillatorAmplitude,
                        amp,
                        samplerate,
                    ),
                    *add,
                    false,
                    false,
                    samplerate,
                ),
            },
        ),
        SynthParameterValue::LinRamp(from, to, time, op) => ValueOrModulator::Mod(
            *from,
            Modulator::lin_ramp(*op, *from, *to, *time, samplerate),
        ),
        SynthParameterValue::LogRamp(from, to, time, op) => ValueOrModulator::Mod(
            *from,
            Modulator::log_ramp(*op, *from, *to, *time, samplerate),
        ),
        SynthParameterValue::ExpRamp(from, to, time, op) => ValueOrModulator::Mod(
            *from,
            Modulator::exp_ramp(*op, *from, *to, *time, samplerate),
        ),
        SynthParameterValue::MultiPointEnvelope(segments, loop_env, op) => {
            // if this is the master envelope, don't pass as a modulator
            // which makes sense only on modulateable parameters
            if let SynthParameterLabel::Envelope = par {
                ValueOrModulator::Val(SynthParameterValue::MultiPointEnvelope(
                    segments.to_vec(),
                    *loop_env,
                    *op,
                ))
            } else {
                let init = if let Some(seg) = segments.first() {
                    seg.from
                } else {
                    0.0
                };
                ValueOrModulator::Mod(
                    init,
                    Modulator::multi_point_envelope(*op, segments.to_vec(), *loop_env, samplerate),
                )
            }
        }
        _ => ValueOrModulator::Val(val.clone()),
    }
}

/// oscillators, the sampler, etc are sources
pub trait MonoSource<const BUFSIZE: usize>: MonoSourceClone<BUFSIZE> {
    fn set_parameter(&mut self, par: SynthParameterLabel, value: &SynthParameterValue);
    fn set_modulator(&mut self, par: SynthParameterLabel, init: f32, modulator: Modulator<BUFSIZE>);

    /// default impl so we have a common interface ...
    fn set_param_or_modulator(
        &mut self,
        par: SynthParameterLabel,
        val_or_mod: ValueOrModulator<BUFSIZE>,
    ) {
        match val_or_mod {
            ValueOrModulator::Val(val) => self.set_parameter(par, &val),
            ValueOrModulator::Mod(init, modulator) => self.set_modulator(par, init, modulator),
        }
    }

    fn finish(&mut self);
    fn reset(&mut self);
    fn is_finished(&self) -> bool;
    fn get_next_block(
        &mut self,
        start_sample: usize,
        in_buffers: &[SampleBuffer],
    ) -> [f32; BUFSIZE];
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

/// so far only for stereo sampler
pub trait StereoSource<const BUFSIZE: usize>: StereoSourceClone<BUFSIZE> {
    fn set_parameter(&mut self, par: SynthParameterLabel, value: &SynthParameterValue);
    fn set_modulator(&mut self, par: SynthParameterLabel, init: f32, modulator: Modulator<BUFSIZE>);

    /// default impl so we have a common interface ...
    fn set_param_or_modulator(
        &mut self,
        par: SynthParameterLabel,
        val_or_mod: ValueOrModulator<BUFSIZE>,
    ) {
        match val_or_mod {
            ValueOrModulator::Val(val) => self.set_parameter(par, &val),
            ValueOrModulator::Mod(init, modulator) => self.set_modulator(par, init, modulator),
        }
    }

    fn finish(&mut self);
    fn reset(&mut self);
    fn is_finished(&self) -> bool;
    fn get_next_block(
        &mut self,
        start_sample: usize,
        in_buffers: &[SampleBuffer],
    ) -> [[f32; BUFSIZE]; 2];
}

pub trait StereoSourceClone<const BUFSIZE: usize> {
    fn clone_box(&self) -> Box<dyn StereoSource<BUFSIZE> + Send + Sync>;
}

impl<const BUFSIZE: usize, T> StereoSourceClone<BUFSIZE> for T
where
    T: 'static + StereoSource<BUFSIZE> + Clone + Send + Sync,
{
    fn clone_box(&self) -> Box<dyn StereoSource<BUFSIZE> + Send + Sync> {
        Box::new(self.clone())
    }
}

impl<const BUFSIZE: usize> Clone for Box<dyn StereoSource<BUFSIZE> + Send + Sync> {
    fn clone(&self) -> Box<dyn StereoSource<BUFSIZE> + Send + Sync> {
        self.clone_box()
    }
}

/// filters etc are effects
pub trait MonoEffect<const BUFSIZE: usize> {
    fn finish(&mut self);
    fn is_finished(&self) -> bool;
    fn set_parameter(&mut self, par: SynthParameterLabel, value: &SynthParameterValue);
    fn set_modulator(&mut self, par: SynthParameterLabel, init: f32, modulator: Modulator<BUFSIZE>);

    /// default impl so we have a common interface ...
    fn set_param_or_modulator(
        &mut self,
        par: SynthParameterLabel,
        val_or_mod: ValueOrModulator<BUFSIZE>,
    ) {
        match val_or_mod {
            ValueOrModulator::Val(val) => self.set_parameter(par, &val),
            ValueOrModulator::Mod(init, modulator) => self.set_modulator(par, init, modulator),
        }
    }

    fn process_block(
        &mut self,
        block: [f32; BUFSIZE],
        start_sample: usize,
        in_buffers: &[SampleBuffer],
    ) -> [f32; BUFSIZE];
}

/// there's a freeverb- and a convolution-based implementation
pub trait MultichannelReverb<const BUFSIZE: usize, const NCHAN: usize> {
    fn set_parameter(&mut self, par: SynthParameterLabel, value: &SynthParameterValue);
    fn set_param_or_modulator(
        &mut self,
        par: SynthParameterLabel,
        val_or_mod: ValueOrModulator<BUFSIZE>,
    ) {
        match val_or_mod {
            ValueOrModulator::Val(val) => self.set_parameter(par, &val),
            ValueOrModulator::Mod(_, _) => {} // no modulators possible so far
        }
    }
    fn process(&mut self, block: [[f32; BUFSIZE]; NCHAN]) -> [[f32; BUFSIZE]; NCHAN];
}

// we need some more info in case a synth can have more than one
// of something ...
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct SynthParameterAddress {
    pub label: SynthParameterLabel,
    // the index is optional, as so far, most synths have only one of each
    // (filter, env), so we can work with that ...
    pub idx: Option<usize>,
}

impl From<SynthParameterLabel> for SynthParameterAddress {
    fn from(label: SynthParameterLabel) -> Self {
        SynthParameterAddress { label, idx: None }
    }
}

/// This is where the building blocks come together
pub trait Synth<const BUFSIZE: usize, const NCHAN: usize> {
    fn set_parameter(&mut self, par: SynthParameterAddress, value: &SynthParameterValue);
    fn set_modulator(
        &mut self,
        par: SynthParameterAddress,
        init: f32,
        modulator: Modulator<BUFSIZE>,
    );

    /// default impl so we have a common interface ...
    fn set_param_or_modulator(
        &mut self,
        par: SynthParameterAddress,
        val_or_mod: ValueOrModulator<BUFSIZE>,
    ) {
        match val_or_mod {
            ValueOrModulator::Val(val) => self.set_parameter(par, &val),
            ValueOrModulator::Mod(init, modulator) => self.set_modulator(par, init, modulator),
        }
    }

    fn finish(&mut self);
    fn is_finished(&self) -> bool;

    fn get_next_block(
        &mut self,
        start_sample: usize,
        in_buffers: &[SampleBuffer],
    ) -> [[f32; BUFSIZE]; NCHAN];

    fn reverb_level(&self) -> f32;
    fn delay_level(&self) -> f32;
}
