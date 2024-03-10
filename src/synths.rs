pub mod ambisonic;
pub mod n_channel;

// channel-based synths
pub use crate::synths::n_channel::n_channel_sampler::NChannelSampler;
pub use crate::synths::n_channel::n_channel_stereo_sampler::NChannelStereoSampler;
pub use crate::synths::n_channel::risset_bell::RissetBell;
pub use crate::synths::n_channel::single_oscillator_synth::SingleOscillatorSynth;

// ambisonic synths
pub use crate::synths::ambisonic::ambisonic_sampler_o1::AmbisonicSamplerO1;

use crate::building_blocks::{FilterType, OscillatorType};

#[repr(C)]
pub enum SynthType {
    Sampler(FilterType, FilterType, FilterType, FilterType),
    AmbisonicSampler(FilterType, FilterType, FilterType, FilterType),
    LiveSampler(FilterType, FilterType, FilterType, FilterType),
    FrozenSampler(FilterType, FilterType, FilterType, FilterType),
    SingleOscillator(OscillatorType, FilterType, FilterType),
    MultiOscillator(Vec<OscillatorType>, FilterType, FilterType),
    KarPlusPlus(OscillatorType, FilterType, FilterType),
    RissetBell,
}
