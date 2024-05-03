use crate::building_blocks::{
    Modulator, MonoEffect, SampleBuffer, SynthParameterLabel, SynthParameterValue,
};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
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
    // user parameters
    mix: f32,
    bits: u32,

    // modulate bitrate ...
    bits_mod: Option<Modulator<BUFSIZE>>,

    stages_buf: [f32; BUFSIZE],
    update_every: usize,
    mode: BitcrusherMode,
}

impl<const BUFSIZE: usize> Bitcrusher<BUFSIZE> {
    pub fn new(mode: BitcrusherMode) -> Self {
        Bitcrusher {
            mix: 1.0,
            bits: 32,
            bits_mod: None,
            stages_buf: [f32::powf(2.0, 31.0); BUFSIZE],
            update_every: 1,
            mode,
        }
    }
}

impl<const BUFSIZE: usize> MonoEffect<BUFSIZE> for Bitcrusher<BUFSIZE> {
    fn set_modulator(
        &mut self,
        par: SynthParameterLabel,
        init: f32,
        modulator: Modulator<BUFSIZE>,
    ) {
        if let SynthParameterLabel::BitcrusherBits = par {
            self.bits = init as u32;
            self.bits_mod = Some(modulator);
        }
    }

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
                self.stages_buf = [f32::powf(2.0, (self.bits - 1) as f32); BUFSIZE];
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
        sample_buffers: &[SampleBuffer],
    ) -> [f32; BUFSIZE] {
        if self.mix == 0.0 {
            block
        } else {
            let mut out_block = [0.0; BUFSIZE];

            if let Some(m) = self.bits_mod.as_mut() {
                let mod_out = m.process(self.bits as f32, 0, sample_buffers);
                for i in 0..BUFSIZE {
                    self.stages_buf[i] = f32::powf(2.0, mod_out[i] - 1.0);
                }
            }

            if self.bits >= 32 {
                for (i, x) in block.iter().enumerate() {
                    if i % self.update_every == 0 {
                        out_block[i] = *x;
                    } else {
                        out_block[i] = out_block[i - 1];
                    }
                }
            } else {
                match self.mode {
                    BitcrusherMode::Cast => {
                        for (i, x) in block.iter().enumerate() {
                            if i % self.update_every == 0 {
                                out_block[i] =
                                    ((*x * self.stages_buf[i]) as i32) as f32 / self.stages_buf[i];
                            } else {
                                out_block[i] = out_block[i - 1];
                            }
                        }
                    }
                    BitcrusherMode::Floor => {
                        for (i, x) in block.iter().enumerate() {
                            if i % self.update_every == 0 {
                                out_block[i] =
                                    (*x * self.stages_buf[i]).floor() / self.stages_buf[i];
                            } else {
                                out_block[i] = out_block[i - 1];
                            }
                        }
                    }
                    BitcrusherMode::Ceil => {
                        for (i, x) in block.iter().enumerate() {
                            if i % self.update_every == 0 {
                                out_block[i] =
                                    (*x * self.stages_buf[i]).ceil() / self.stages_buf[i];
                            } else {
                                out_block[i] = out_block[i - 1];
                            }
                        }
                    }
                    BitcrusherMode::Round => {
                        for (i, x) in block.iter().enumerate() {
                            if i % self.update_every == 0 {
                                out_block[i] =
                                    (*x * self.stages_buf[i]).round() / self.stages_buf[i];
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
