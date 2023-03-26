use crate::building_blocks::{
    Modulator, MonoEffect, SampleBuffer, SynthParameterLabel, SynthParameterValue,
};

/**
 * naive, simple, cubic digital waveshaping distortion
 */
pub struct Waveshaper<const BUFSIZE: usize> {
    mix: f32,
}

impl<const BUFSIZE: usize> Waveshaper<BUFSIZE> {
    pub fn new() -> Self {
        Waveshaper { mix: 0.0 }
    }
}

impl<const BUFSIZE: usize> Default for Waveshaper<BUFSIZE> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const BUFSIZE: usize> MonoEffect<BUFSIZE> for Waveshaper<BUFSIZE> {
    fn set_modulator(&mut self, _: SynthParameterLabel, _: f32, _: Modulator<BUFSIZE>) {}

    fn set_parameter(&mut self, label: SynthParameterLabel, value: &SynthParameterValue) {
        if let SynthParameterLabel::WaveshaperMix = label {
            if let SynthParameterValue::ScalarF32(val) = value {
                self.mix = *val;
            }
        }
    }
    fn finish(&mut self) {} // this effect is stateless
    fn is_finished(&self) -> bool {
        false
    } // it's never finished ..

    // start sample isn't really needed either ...
    fn process_block(
        &mut self,
        block: [f32; BUFSIZE],
        _: usize,
        _: &[SampleBuffer],
    ) -> [f32; BUFSIZE] {
        if self.mix == 0.0 {
            block
        } else {
            let mut out_block = [0.0; BUFSIZE];
            for (i, x) in block.iter().enumerate() {
                let y = if *x > 1.0 {
                    1.0
                } else if *x < -1.0 {
                    -1.0
                } else {
                    1.5 * x - 0.5 * x * x * x
                };
                out_block[i] = ((1.0 - self.mix) * x) + (self.mix * y);
            }
            out_block
        }
    }
}
