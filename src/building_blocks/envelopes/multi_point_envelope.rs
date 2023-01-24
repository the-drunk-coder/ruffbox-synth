use crate::building_blocks::{
    envelopes::source_env::MultiPointEnvelope, EnvelopeSegmentInfo, Modulator, MonoEffect,
    MonoSource, SampleBuffer, SynthParameterLabel, SynthParameterValue,
};

/// more complex, configurable envelope
#[derive(Clone)]
pub struct MultiPointEffectEnvelope<const BUFSIZE: usize> {
    inner_env: MultiPointEnvelope<BUFSIZE>,
}

impl<const BUFSIZE: usize> MultiPointEffectEnvelope<BUFSIZE> {
    pub fn new(segment_infos: Vec<EnvelopeSegmentInfo>, loop_env: bool, samplerate: f32) -> Self {
        MultiPointEffectEnvelope {
            inner_env: MultiPointEnvelope::new(segment_infos, loop_env, samplerate),
        }
    }
    pub fn empty(samplerate: f32) -> Self {
        MultiPointEffectEnvelope {
            inner_env: MultiPointEnvelope::empty(samplerate),
        }
    }
}

impl<const BUFSIZE: usize> MonoEffect<BUFSIZE> for MultiPointEffectEnvelope<BUFSIZE> {
    fn finish(&mut self) {
        self.inner_env.finish();
    }

    fn is_finished(&self) -> bool {
        self.inner_env.is_finished()
    }

    fn set_modulator(
        &mut self,
        label: SynthParameterLabel,
        init: f32,
        modulator: Modulator<BUFSIZE>,
    ) {
        self.inner_env.set_modulator(label, init, modulator);
    }

    /// multi-point envelopes can only be set as a whole,
    /// the parameter is just passed on to the inner envelope ...
    fn set_parameter(&mut self, par: SynthParameterLabel, value: &SynthParameterValue) {
        self.inner_env.set_parameter(par, value);
    }

    fn process_block(
        &mut self,
        block: [f32; BUFSIZE],
        start_sample: usize,
        bufs: &[SampleBuffer],
    ) -> [f32; BUFSIZE] {
        let mut out: [f32; BUFSIZE] = [0.0; BUFSIZE];
        let env = self.inner_env.get_next_block(start_sample, bufs);
        for i in start_sample..BUFSIZE {
            out[i] = block[i] * env[i];
        }
        out
    }
}
