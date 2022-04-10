pub mod lf_cub_synth;
pub mod lf_saw_synth;
pub mod lf_square_synth;
pub mod lf_tri_synth;
pub mod n_channel_sampler;
pub mod risset_bell;
/// a collection of pre-fabricated synths
pub mod sine_synth;

pub use crate::ruffbox::synth::synths::n_channel::lf_cub_synth::LFCubSynth;
pub use crate::ruffbox::synth::synths::n_channel::lf_saw_synth::LFSawSynth;
pub use crate::ruffbox::synth::synths::n_channel::lf_square_synth::LFSquareSynth;
pub use crate::ruffbox::synth::synths::n_channel::lf_tri_synth::LFTriSynth;
pub use crate::ruffbox::synth::synths::n_channel::n_channel_sampler::NChannelSampler;
pub use crate::ruffbox::synth::synths::n_channel::risset_bell::RissetBell;
pub use crate::ruffbox::synth::synths::n_channel::sine_synth::SineSynth;
