use crate::building_blocks::{
    Modulator, MonoEffect, SynthParameterLabel, SynthParameterValue,
};

/**
 * dummy filter ... so unnecessary filters aren't in the way ...
 */
pub struct DummyFilter<const BUFSIZE: usize> {}

impl<const BUFSIZE: usize> DummyFilter<BUFSIZE> {
    pub fn new() -> Self {
        DummyFilter {}
    }
}

impl<const BUFSIZE: usize> Default for DummyFilter<BUFSIZE> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const BUFSIZE: usize> MonoEffect<BUFSIZE> for DummyFilter<BUFSIZE> {
    
    fn set_modulator(&mut self, _: SynthParameterLabel, _: f32, _: Modulator<BUFSIZE>) {}

    fn set_parameter(&mut self, _: SynthParameterLabel, _: &SynthParameterValue) {}
    fn finish(&mut self) {} // this effect is stateless
    fn is_finished(&self) -> bool {
        false
    } // it's never finished ..

    // start sample isn't really needed either ...
    fn process_block(&mut self, block: [f32; BUFSIZE], _: usize, _: &[Vec<f32>]) -> [f32; BUFSIZE] {
        block
    }
}
