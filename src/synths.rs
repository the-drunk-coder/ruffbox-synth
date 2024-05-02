pub mod ambisonic;
pub mod n_channel;

// channel-based synths
pub use crate::synths::n_channel::n_channel_sampler::NChannelSampler;
pub use crate::synths::n_channel::n_channel_stereo_sampler::NChannelStereoSampler;
pub use crate::synths::n_channel::risset_bell::RissetBell;
pub use crate::synths::n_channel::single_oscillator_synth::SingleOscillatorSynth;

// ambisonic synths
pub use crate::synths::ambisonic::ambisonic_sampler_o1::AmbisonicSamplerO1;

use crate::building_blocks::{EffectType, FilterType, OscillatorType};

/// parts to assemble a synth
#[repr(C)]
pub struct SynthDescription {
    /// effects before the filter ...
    pub pre_filter_effects: Vec<EffectType>,

    /// filter chain (keeping these apart for now ...)
    pub filters: Vec<FilterType>,

    /// leave empty if not needed ...
    pub oscillator_types: Vec<OscillatorType>,
}

#[repr(C)]
pub enum SynthType {
    Sampler(SynthDescription),
    AmbisonicSampler(SynthDescription),
    LiveSampler(SynthDescription),
    FrozenSampler(SynthDescription),
    SingleOscillator(SynthDescription),
    MultiOscillator(SynthDescription),
    KarPlusPlus(SynthDescription),
    RissetBell,
}
