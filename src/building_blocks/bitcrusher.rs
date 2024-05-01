use crate::building_blocks::{
    Modulator, MonoEffect, SampleBuffer, SynthParameterLabel, SynthParameterValue,
};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
/// different modes for the bitcrusher effect
pub enum BitcrusherMode {
    Cast,  // quantize by casting to i32
    Floor, // quantize by floor operation
    Ceil,  // quantize by ceil operation
    Round, // quantize by round operation
}

/**
 * naive, simple, cubic bitcrusher/downsampler
 */
pub struct Bitcrusher<const BUFSIZE: usize> {
    mix: f32,
    bits: u32,
    stages: f32,
    update_every: usize,
    mode: BitcrusherMode,
}

impl<const BUFSIZE: usize> Bitcrusher<BUFSIZE> {
    pub fn new(_sr: f32) -> Self {
        Bitcrusher {
            mix: 0.0,
            bits: 32,
            stages: f32::powf(2.0, 31.0),
            update_every: 1,
            mode: BitcrusherMode::Cast,
        }
    }
}

impl<const BUFSIZE: usize> MonoEffect<BUFSIZE> for Bitcrusher<BUFSIZE> {
    fn set_modulator(&mut self, _: SynthParameterLabel, _: f32, _: Modulator<BUFSIZE>) {}

    fn set_parameter(&mut self, label: SynthParameterLabel, value: &SynthParameterValue) {
        if let SynthParameterLabel::BitcrusherMix = label {
            if let SynthParameterValue::ScalarF32(val) = value {
                self.mix = *val;
            }
        }
        if let SynthParameterLabel::BitcrusherBits = label {
            if let SynthParameterValue::ScalarF32(val) = value {
                self.bits = *val as u32;
                if self.bits < 1 {
                    self.bits = 1;
                }
                self.stages = f32::powf(2.0, (self.bits - 1) as f32);
            }
        }
        if let SynthParameterLabel::BitcrusherDownsampling = label {
            if let SynthParameterValue::ScalarF32(val) = value {
                self.update_every = *val as usize;
                if self.update_every < 1 {
                    self.update_every = 1;
                }
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

            if self.bits >= 32 {
                for (i, x) in block.iter().enumerate() {
                    out_block[i] = *x;
                }
            } else {
                match self.mode {
                    BitcrusherMode::Cast => {
                        for (i, x) in block.iter().enumerate() {
                            if i % self.update_every == 0 {
                                out_block[i] = ((*x * self.stages) as i32) as f32 / self.stages;
                            } else {
                                out_block[i] = out_block[i - 1];
                            }
                        }
                    }
                    BitcrusherMode::Floor => {
                        for (i, x) in block.iter().enumerate() {
                            if i % self.update_every == 0 {
                                out_block[i] = (*x * self.stages).floor() / self.stages;
                            } else {
                                out_block[i] = out_block[i - 1];
                            }
                        }
                    }
                    BitcrusherMode::Ceil => {
                        for (i, x) in block.iter().enumerate() {
                            if i % self.update_every == 0 {
                                out_block[i] = (*x * self.stages).ceil() / self.stages;
                            } else {
                                out_block[i] = out_block[i - 1];
                            }
                        }
                    }
                    BitcrusherMode::Round => {
                        for (i, x) in block.iter().enumerate() {
                            if i % self.update_every == 0 {
                                out_block[i] = (*x * self.stages).round() / self.stages;
                            } else {
                                out_block[i] = out_block[i - 1];
                            }
                        }
                    }
                }
            }

            out_block
        }
    }
}
