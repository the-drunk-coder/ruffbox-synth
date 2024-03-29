// a collection of pre-fabricated synths
pub mod karplusplus;
pub mod multi_oscillator_synth;
pub mod n_channel_sampler;
pub mod n_channel_stereo_sampler;
pub mod risset_bell;
pub mod single_oscillator_synth;

pub use crate::synths::n_channel::karplusplus::KarPlusPlus;
pub use crate::synths::n_channel::multi_oscillator_synth::MultiOscillatorSynth;
pub use crate::synths::n_channel::n_channel_sampler::NChannelSampler;
pub use crate::synths::n_channel::risset_bell::RissetBell;
pub use crate::synths::n_channel::single_oscillator_synth::SingleOscillatorSynth;
