use crate::building_blocks::{Modulator, SynthParameterLabel, SynthParameterValue};

use std::f32::consts::PI;

pub struct PanChan<const BUFSIZE: usize, const NCHAN: usize> {
    levels: [[f32; BUFSIZE]; NCHAN],
    pos_mod: Option<Modulator<BUFSIZE>>,
    pos: f32,
    samplerate: f32,
}

impl<const BUFSIZE: usize, const NCHAN: usize> PanChan<BUFSIZE, NCHAN> {
    pub fn new(sr: f32) -> Self {
        let mut lvls = [[0.0; BUFSIZE]; NCHAN];
        lvls[0] = [1.0; BUFSIZE];
        // always start on first channel
        PanChan {
            levels: lvls,
            pos_mod: None,
            pos: 0.0,
            samplerate: sr,
        }
    }

    /// Set the parameter for this panner.
    pub fn set_parameter(&mut self, par: SynthParameterLabel, value: &SynthParameterValue) {
        // if it was more parameters, match would be better,
        // but this way clippy doesn't complain
        if par == SynthParameterLabel::ChannelPosition {
            match value {
                SynthParameterValue::ScalarF32(p) => {
                    self.pos = *p; // keep for later

                    self.levels = [[0.0; BUFSIZE]; NCHAN];

                    let lower = p.floor();
                    let angle_rad = (p - lower) * PI * 0.5;
                    let upper = lower + 1.0;

                    self.levels[lower as usize % (NCHAN as usize)] = [angle_rad.cos(); BUFSIZE];
                    self.levels[upper as usize % (NCHAN as usize)] = [angle_rad.sin(); BUFSIZE];
                }
                SynthParameterValue::Lfo(init, freq, amp, add, op) => {
                    self.pos = *init; // keep for later
                    self.pos_mod = Some(Modulator::lfo(*op, *freq, *amp, *add, self.samplerate))
                }
                SynthParameterValue::LFTri(init, freq, amp, add, op) => {
                    self.pos = *init; // keep for later
                    self.pos_mod = Some(Modulator::lftri(*op, *freq, *amp, *add, self.samplerate))
                }
                SynthParameterValue::LFSaw(init, freq, amp, add, op) => {
                    self.pos = *init; // keep for later
                    self.pos_mod = Some(Modulator::lfsaw(*op, *freq, *amp, *add, self.samplerate))
                }
                SynthParameterValue::LFSquare(init, freq, pw, amp, add, op) => {
                    self.pos = *init; // keep for later
                    self.pos_mod = Some(Modulator::lfsquare(
                        *op,
                        *pw,
                        *freq,
                        *amp,
                        *add,
                        self.samplerate,
                    ))
                }
                _ => {}
            }
        }
    }

    /// pan mono to stereo
    #[allow(clippy::needless_range_loop)]
    pub fn process_block(
        &mut self,
        block: [f32; BUFSIZE],
        start_sample: usize,
        sample_buffers: &[Vec<f32>],
    ) -> [[f32; BUFSIZE]; NCHAN] {
        if self.pos_mod.is_some() {
            self.levels = [[0.0; BUFSIZE]; NCHAN];
            let pos_buf =
                self.pos_mod
                    .as_mut()
                    .unwrap()
                    .process(self.pos, start_sample, sample_buffers);
            for (idx, p) in pos_buf.iter().enumerate() {
                let lower = p.floor();
                let angle_rad = (p - lower) * PI * 0.5;
                let upper = lower + 1.0;

                self.levels[lower as usize % (NCHAN as usize)][idx] = angle_rad.cos();
                self.levels[upper as usize % (NCHAN as usize)][idx] = angle_rad.sin();
            }
        }

        // I think the range loop is way more intuitive and easy to read here ...
        let mut out_buf = [[0.0; BUFSIZE]; NCHAN];
        for c in 0..NCHAN {
            for s in 0..BUFSIZE {
                out_buf[c][s] = block[s] * self.levels[c][s];
            }
        }
        out_buf
    }
}

// TEST TEST TEST
#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn panchan_test_basic_pan() {
        let mut pchan = PanChan::<128, 2>::new(44100.0);

        let mut block = [0.0; 128];
        block[0] = 1.0;

        pchan.set_parameter(
            SynthParameterLabel::ChannelPosition,
            &SynthParameterValue::ScalarF32(0.5),
        );

        let block_out = pchan.process_block(block, 0, &Vec::new());

        assert_approx_eq::assert_approx_eq!(block_out[0][0], 0.707, 0.001);
        assert_approx_eq::assert_approx_eq!(block_out[1][0], 0.707, 0.001);
    }

    #[test]
    fn panchan_test_left_pan() {
        let mut pchan = PanChan::<128, 2>::new(44100.0);

        pchan.set_parameter(
            SynthParameterLabel::ChannelPosition,
            &SynthParameterValue::ScalarF32(0.0),
        );

        let mut block = [0.0; 128];
        block[0] = 1.0;

        let block_out = pchan.process_block(block, 0, &Vec::new());

        assert_approx_eq::assert_approx_eq!(block_out[0][0], 1.0, 0.0001);
        assert_approx_eq::assert_approx_eq!(block_out[1][0], 0.0, 0.0001);
    }

    #[test]
    fn panchan_test_right_pan() {
        let mut pchan = PanChan::<128, 2>::new(44100.0);

        pchan.set_parameter(
            SynthParameterLabel::ChannelPosition,
            &SynthParameterValue::ScalarF32(1.0),
        );

        let mut block = [0.0; 128];
        block[0] = 1.0;

        let block_out = pchan.process_block(block, 0, &Vec::new());

        assert_approx_eq::assert_approx_eq!(block_out[0][0], 0.0, 0.0001);
        assert_approx_eq::assert_approx_eq!(block_out[1][0], 1.0, 0.0001);
    }

    #[test]
    fn panchan_test_multi() {
        let mut pchan = PanChan::<128, 8>::new(44100.0);

        let mut block = [0.0; 128];
        block[0] = 1.0;

        pchan.set_parameter(
            SynthParameterLabel::ChannelPosition,
            &SynthParameterValue::ScalarF32(6.0),
        );
        let mut block_out = pchan.process_block(block, 0, &Vec::new());

        assert_approx_eq::assert_approx_eq!(block_out[0][0], 0.0, 0.0001);
        assert_approx_eq::assert_approx_eq!(block_out[1][0], 0.0, 0.0001);
        assert_approx_eq::assert_approx_eq!(block_out[2][0], 0.0, 0.0001);
        assert_approx_eq::assert_approx_eq!(block_out[3][0], 0.0, 0.0001);
        assert_approx_eq::assert_approx_eq!(block_out[4][0], 0.0, 0.0001);
        assert_approx_eq::assert_approx_eq!(block_out[5][0], 0.0, 0.0001);
        assert_approx_eq::assert_approx_eq!(block_out[6][0], 1.0, 0.0001);
        assert_approx_eq::assert_approx_eq!(block_out[7][0], 0.0, 0.0001);

        pchan.set_parameter(
            SynthParameterLabel::ChannelPosition,
            &SynthParameterValue::ScalarF32(2.0),
        );
        block_out = pchan.process_block(block, 0, &Vec::new());

        assert_approx_eq::assert_approx_eq!(block_out[0][0], 0.0, 0.0001);
        assert_approx_eq::assert_approx_eq!(block_out[1][0], 0.0, 0.0001);
        assert_approx_eq::assert_approx_eq!(block_out[2][0], 1.0, 0.0001);
        assert_approx_eq::assert_approx_eq!(block_out[3][0], 0.0, 0.0001);
        assert_approx_eq::assert_approx_eq!(block_out[4][0], 0.0, 0.0001);
        assert_approx_eq::assert_approx_eq!(block_out[5][0], 0.0, 0.0001);
        assert_approx_eq::assert_approx_eq!(block_out[6][0], 0.0, 0.0001);
        assert_approx_eq::assert_approx_eq!(block_out[7][0], 0.0, 0.0001);

        pchan.set_parameter(
            SynthParameterLabel::ChannelPosition,
            &SynthParameterValue::ScalarF32(0.0),
        );
        block_out = pchan.process_block(block, 0, &Vec::new());

        assert_approx_eq::assert_approx_eq!(block_out[0][0], 1.0, 0.0001);
        assert_approx_eq::assert_approx_eq!(block_out[1][0], 0.0, 0.0001);
        assert_approx_eq::assert_approx_eq!(block_out[2][0], 0.0, 0.0001);
        assert_approx_eq::assert_approx_eq!(block_out[3][0], 0.0, 0.0001);
        assert_approx_eq::assert_approx_eq!(block_out[4][0], 0.0, 0.0001);
        assert_approx_eq::assert_approx_eq!(block_out[5][0], 0.0, 0.0001);
        assert_approx_eq::assert_approx_eq!(block_out[6][0], 0.0, 0.0001);
        assert_approx_eq::assert_approx_eq!(block_out[7][0], 0.0, 0.0001);

        pchan.set_parameter(
            SynthParameterLabel::ChannelPosition,
            &SynthParameterValue::ScalarF32(8.0),
        );
        block_out = pchan.process_block(block, 0, &Vec::new());

        assert_approx_eq::assert_approx_eq!(block_out[0][0], 1.0, 0.0001);
        assert_approx_eq::assert_approx_eq!(block_out[1][0], 0.0, 0.0001);
        assert_approx_eq::assert_approx_eq!(block_out[2][0], 0.0, 0.0001);
        assert_approx_eq::assert_approx_eq!(block_out[3][0], 0.0, 0.0001);
        assert_approx_eq::assert_approx_eq!(block_out[4][0], 0.0, 0.0001);
        assert_approx_eq::assert_approx_eq!(block_out[5][0], 0.0, 0.0001);
        assert_approx_eq::assert_approx_eq!(block_out[6][0], 0.0, 0.0001);
        assert_approx_eq::assert_approx_eq!(block_out[7][0], 0.0, 0.0001);
    }
}
