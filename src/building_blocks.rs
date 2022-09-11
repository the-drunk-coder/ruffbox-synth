pub mod ambisonics;
pub mod convolver;
pub mod delay;
pub mod envelopes;
pub mod filters;
pub mod interpolation;
pub mod mod_env;
pub mod modulator;
pub mod oscillators;
pub mod reverb;
pub mod routing;
pub mod sampler;

pub use crate::building_blocks::modulator::Modulator;

#[repr(C)]
#[derive(Clone, Copy)]
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
}


#[repr(C)]
#[derive(Clone, Copy)]
pub enum FilterType {
    Dummy,
    Lpf18,
    BiquadHpf12dB,
    BiquadLpf12dB,
    BiquadHpf24dB,
    BiquadLpf24dB,
    PeakEQ,
}

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
    HighpassFilterType,       //
    EnvelopeLevel,            // 12
    OscillatorAmplitude,      // 13 (oscillator amplitude)
    OscillatorPhaseRelative,  // 14 (radians)
    OscillatorPhaseEffective, // 15 (the target value or effective offset dependent on the amplitude)
    LowpassCutoffFrequency,   // 16
    LowpassQFactor,           // 17
    LowpassFilterDistortion,  // 18
    LowpassFilterType,        //
    PeakFrequency,            // 19
    PeakGain,                 // 20
    PeakBandwidth,            // 21
    Peak1Frequency,           // 19
    Peak1Gain,                // 20
    Peak1Bandwidth,           // 21
    Peak2Frequency,           // 19
    Peak2Gain,                // 20
    Peak2Bandwidth,           // 21
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

// from an outside perspective, there can be modulator-valued parameters (like, an lfo-valued parameter)
#[derive(Clone)]
#[rustfmt::skip]
pub enum SynthParameterValue {
    ScalarF32(f32),
    ScalarU32(u32),
    ScalarUsize(usize),
    VecF32(Vec<f32>),
    FilterType(FilterType), // these aren't really treated as parameters so far, but as a pragmatic solition that's ok for now ...
    MatrixF32((usize, usize), Vec<Vec<f32>>), // dimension, content
    // lfo param order - init val, freq, phase, amp, add, operation (mul, add, sub, div, replace)
    Lfo(f32, Box<SynthParameterValue>, f32, Box<SynthParameterValue>, f32, ValOp), // sine lfo
    LFSaw(f32, Box<SynthParameterValue>, f32, Box<SynthParameterValue>, f32, ValOp), // sawtooth lfo
    LFRSaw(f32, Box<SynthParameterValue>, f32, Box<SynthParameterValue>, f32, ValOp), // reverse sawtooth lfo
    LFTri(f32, Box<SynthParameterValue>, f32, Box<SynthParameterValue>, f32, ValOp), // triangle wave lfo
    LFSquare(f32, Box<SynthParameterValue>, f32, Box<SynthParameterValue>, f32, ValOp), // squarewave lfo
    LinRamp(f32, f32, f32, ValOp), // linear ramp - from, to, time
    LogRamp(f32, f32, f32, ValOp), // logarithmic ramp - from, to, time
    ExpRamp(f32, f32, f32, ValOp), // exponential ramp - from, to, time
    MultiPointEnvelope(Vec<mod_env::SegmentInfo>, bool, ValOp), // segments, loop ...
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
        _ => ValueOrModulator::Val(val.clone()),
    }
}

/// oscillators, the sampler, etc are sources
pub trait MonoSource<const BUFSIZE: usize>: MonoSourceClone<BUFSIZE> {
    fn set_parameter(&mut self, par: SynthParameterLabel, value: &SynthParameterValue);
    fn set_modulator(&mut self, par: SynthParameterLabel, init: f32, modulator: Modulator<BUFSIZE>);
    fn set_param_or_modulator(
        &mut self,
        par: SynthParameterLabel,
        val_or_mod: ValueOrModulator<BUFSIZE>,
    );
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
    fn set_param_or_modulator(
        &mut self,
        par: SynthParameterLabel,
        val_or_mod: ValueOrModulator<BUFSIZE>,
    );
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
    fn set_param_or_modulator(
        &mut self,
        par: SynthParameterLabel,
        val_or_mod: ValueOrModulator<BUFSIZE>,
    );
    fn process(&mut self, block: [[f32; BUFSIZE]; NCHAN]) -> [[f32; BUFSIZE]; NCHAN];
}

/// This is where the building blocks come together
pub trait Synth<const BUFSIZE: usize, const NCHAN: usize> {
    fn set_parameter(&mut self, par: SynthParameterLabel, value: &SynthParameterValue);
    fn set_modulator(&mut self, par: SynthParameterLabel, init: f32, modulator: Modulator<BUFSIZE>);
    fn set_param_or_modulator(
        &mut self,
        par: SynthParameterLabel,
        val_or_mod: ValueOrModulator<BUFSIZE>,
    );
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
