pub mod ambisonics;
pub mod convolution_reverb;
pub mod convolver;
pub mod delay;
pub mod envelopes;
pub mod filters;
pub mod freeverb;
pub mod oscillators;
pub mod routing;
pub mod sampler;
pub mod synths;

#[derive(Clone, Copy)]
pub enum SynthState {
    Fresh,
    Finished,
}

/// a collection of common parameters
#[derive(Clone, Copy)]
pub enum SynthParameter {
    Attack(f32),
    Decay(f32),
    DelayDampeningFrequency(f32),
    DelayFeedback(f32),
    DelayMix(f32),
    DelayTime(f32),
    Duration(f32),
    PitchFrequency(f32),
    PitchNote(u32), // midi note ? it's kinda unused as the frequencies are calculated beforehand
    HighpassCutoffFrequency(f32),
    HighpassQFactor(f32),
    Level(f32),
    LowpassCutoffFrequency(f32),
    LowpassQFactor(f32),
    LowpassFilterDistortion(f32),
    PeakFrequency(f32),
    PeakGain(f32),
    PeakQFactor(f32),
    Pulsewidth(f32),
    PlaybackRate(f32),
    PlaybackStart(f32),
    PlaybackLoop(f32),
    Release(f32),
    ReverbDampening(f32),
    ReverbMix(f32),
    ReverbRoomsize(f32),
    SampleBufferNumber(f32),
    Samplerate(f32),
    ChannelPosition(f32),
    AmbisonicAzimuth(f32),
    AmbisonicElevation(f32),
    Sustain(f32),
}

#[repr(C)]
pub enum SourceType {
    Sampler,
    LiveSampler,
    FrozenSampler,
    SineOsc,
    SineSynth,
    LFCubSynth,
    LFSawSynth,
    LFSquareSynth,
    LFTriangleSynth,
    RissetBell,
}

pub trait MonoSource<const BUFSIZE: usize> {
    fn set_parameter(&mut self, par: SynthParameter);
    fn finish(&mut self);
    fn is_finished(&self) -> bool;
    fn get_next_block(&mut self, start_sample: usize, in_buffers: &[Vec<f32>]) -> [f32; BUFSIZE];
}

pub trait MonoEffect<const BUFSIZE: usize> {
    fn finish(&mut self);
    fn is_finished(&self) -> bool;
    fn set_parameter(&mut self, par: SynthParameter);
    fn process_block(&mut self, block: [f32; BUFSIZE], start_sample: usize) -> [f32; BUFSIZE];
}

pub trait MultichannelReverb<const BUFSIZE: usize, const NCHAN: usize> {
    fn set_parameter(&mut self, par: SynthParameter);
    fn process(&mut self, block: [[f32; BUFSIZE]; NCHAN]) -> [[f32; BUFSIZE]; NCHAN];
}

pub trait Synth<const BUFSIZE: usize, const NCHAN: usize> {
    fn set_parameter(&mut self, par: SynthParameter);
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
