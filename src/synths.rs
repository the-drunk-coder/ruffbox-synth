pub mod n_channel;
pub mod ambisonic;

// channel-based synths
pub use crate::synths::n_channel::lf_cub_synth::LFCubSynth;
pub use crate::synths::n_channel::lf_saw_synth::LFSawSynth;
pub use crate::synths::n_channel::lf_square_synth::LFSquareSynth;
pub use crate::synths::n_channel::lf_tri_synth::LFTriSynth;
pub use crate::synths::n_channel::n_channel_sampler::NChannelSampler;
pub use crate::synths::n_channel::risset_bell::RissetBell;
pub use crate::synths::n_channel::sine_synth::SineSynth;
pub use crate::synths::n_channel::wavetable_synth::WavetableSynth;

// ambisonic synths
pub use crate::synths::ambisonic::ambisonic_sampler_o1::AmbisonicSamplerO1;
pub use crate::synths::ambisonic::lf_saw_synth::LFSawSynth;
pub use crate::synths::ambisonic::lf_square_synth::LFSquareSynth;
pub use crate::synths::ambisonic::lf_tri_synth::LFTriSynth;
pub use crate::synths::ambisonic::sine_synth::SineSynth;
