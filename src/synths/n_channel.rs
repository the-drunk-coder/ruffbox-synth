// a collection of pre-fabricated synths

pub mod lf_cub_synth;
pub mod lf_saw_synth;
pub mod lf_square_synth;
pub mod lf_tri_synth;
pub mod n_channel_sampler;
pub mod risset_bell;
pub mod sine_synth;
pub mod wavematrix_synth;
pub mod wavetable_synth;

pub mod fm_saw_synth;
pub mod fm_square_synth;
pub mod wt_saw_synth;

pub use crate::synths::n_channel::lf_cub_synth::LFCubSynth;
pub use crate::synths::n_channel::lf_saw_synth::LFSawSynth;
pub use crate::synths::n_channel::lf_square_synth::LFSquareSynth;
pub use crate::synths::n_channel::lf_tri_synth::LFTriSynth;
pub use crate::synths::n_channel::n_channel_sampler::NChannelSampler;
pub use crate::synths::n_channel::risset_bell::RissetBell;
pub use crate::synths::n_channel::sine_synth::SineSynth;
pub use crate::synths::n_channel::wavematrix_synth::WavematrixSynth;
pub use crate::synths::n_channel::wavetable_synth::WavetableSynth;

pub use crate::synths::n_channel::wt_saw_synth::WTSawSynth;

pub use crate::synths::n_channel::fm_saw_synth::FMSawSynth;
pub use crate::synths::n_channel::fm_square_synth::FMSquareSynth;
