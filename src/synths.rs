pub mod ambisonic;
pub mod n_channel;

// channel-based synths
pub use crate::synths::n_channel::n_channel_sampler::NChannelSampler;
pub use crate::synths::n_channel::risset_bell::RissetBell;
pub use crate::synths::n_channel::single_oscillator_synth::SingleOscillatorSynth;

// ambisonic synths
pub use crate::synths::ambisonic::ambisonic_sampler_o1::AmbisonicSamplerO1;
pub use crate::synths::ambisonic::lf_saw_synth::LFSawSynth as AmbisonicLFSawSynth;
pub use crate::synths::ambisonic::lf_square_synth::LFSquareSynth as AmbisonicLFSquareSynth;
pub use crate::synths::ambisonic::lf_tri_synth::LFTriSynth as AmbisonicLFTriSynth;
pub use crate::synths::ambisonic::sine_synth::SineSynth as AmbisonicSineSynth;

use crate::building_blocks::FilterType;

#[repr(C)]
pub enum SynthType {
    Sampler(FilterType, FilterType, FilterType, FilterType),
    LiveSampler(FilterType, FilterType, FilterType, FilterType),
    FrozenSampler(FilterType, FilterType, FilterType, FilterType),
    SineSynth(FilterType, FilterType),
    LFCubSynth(FilterType, FilterType),
    LFSawSynth(FilterType, FilterType),
    FMSawSynth(FilterType, FilterType),
    FMSquareSynth(FilterType, FilterType),
    FMTriSynth(FilterType, FilterType),
    LFSquareSynth(FilterType, FilterType),
    LFTriangleSynth(FilterType, FilterType),
    RissetBell,
    Wavetable(FilterType, FilterType),
    Wavematrix(FilterType, FilterType),
    WTSawSynth(FilterType, FilterType),
}
