use crate::ruffbox::synth::{SynthParameterLabel, SynthParameterValue};
use std::f32::consts::PI;

pub struct PanChan<const BUFSIZE: usize, const NCHAN: usize> {
    levels: [f32; NCHAN],
}

impl<const BUFSIZE: usize, const NCHAN: usize> Default for PanChan<BUFSIZE, NCHAN> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const BUFSIZE: usize, const NCHAN: usize> PanChan<BUFSIZE, NCHAN> {
    pub fn new() -> Self {
        let mut lvls = [0.0; NCHAN];
        lvls[0] = 1.0;
        // always start on first channel
        PanChan { levels: lvls }
    }

    /// Set the parameter for this panner.
    pub fn set_parameter(&mut self, par: SynthParameterLabel, value: &SynthParameterValue) {
        // if it was more parameters, match would be better,
        // but this way clippy doesn't complain
        if par == SynthParameterLabel::ChannelPosition {
            if let SynthParameterValue::ScalarF32(p) = value {
                let mut lvls = [0.0; NCHAN];

                let lower = p.floor();
                let angle_rad = (p - lower) * PI * 0.5;

                let upper = lower + 1.0;

                lvls[lower as usize % (NCHAN as usize)] = angle_rad.cos();
                lvls[upper as usize % (NCHAN as usize)] = angle_rad.sin();

                self.levels = lvls;
            }
        }
    }

    /// pan mono to stereo
    #[allow(clippy::needless_range_loop)]
    pub fn process_block(&mut self, block: [f32; BUFSIZE]) -> [[f32; BUFSIZE]; NCHAN] {
        // I think the range loop is way more intuitive and easy to read here ...
        let mut out_buf = [[0.0; BUFSIZE]; NCHAN];
        for c in 0..NCHAN {
            for s in 0..BUFSIZE {
                out_buf[c][s] = block[s] * self.levels[c];
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
        let mut pchan = PanChan::<128, 2>::new();

        let mut block = [0.0; 128];
        block[0] = 1.0;

        pchan.set_parameter(SynthParameterLabel::ChannelPosition, 0.5);

        let block_out = pchan.process_block(block);

        assert_approx_eq::assert_approx_eq!(block_out[0][0], 0.707, 0.001);
        assert_approx_eq::assert_approx_eq!(block_out[1][0], 0.707, 0.001);
    }

    #[test]
    fn panchan_test_left_pan() {
        let mut pchan = PanChan::<128, 2>::new();

        pchan.set_parameter(SynthParameterLabel::ChannelPosition, 0.0);

        let mut block = [0.0; 128];
        block[0] = 1.0;

        let block_out = pchan.process_block(block);

        assert_approx_eq::assert_approx_eq!(block_out[0][0], 1.0, 0.0001);
        assert_approx_eq::assert_approx_eq!(block_out[1][0], 0.0, 0.0001);
    }

    #[test]
    fn panchan_test_right_pan() {
        let mut pchan = PanChan::<128, 2>::new();

        pchan.set_parameter(SynthParameterLabel::ChannelPosition, 1.0);

        let mut block = [0.0; 128];
        block[0] = 1.0;

        let block_out = pchan.process_block(block);

        assert_approx_eq::assert_approx_eq!(block_out[0][0], 0.0, 0.0001);
        assert_approx_eq::assert_approx_eq!(block_out[1][0], 1.0, 0.0001);
    }

    #[test]
    fn panchan_test_multi() {
        let mut pchan = PanChan::<128, 8>::new();

        let mut block = [0.0; 128];
        block[0] = 1.0;

        pchan.set_parameter(SynthParameterLabel::ChannelPosition, 6.0);
        let mut block_out = pchan.process_block(block);

        assert_approx_eq::assert_approx_eq!(block_out[0][0], 0.0, 0.0001);
        assert_approx_eq::assert_approx_eq!(block_out[1][0], 0.0, 0.0001);
        assert_approx_eq::assert_approx_eq!(block_out[2][0], 0.0, 0.0001);
        assert_approx_eq::assert_approx_eq!(block_out[3][0], 0.0, 0.0001);
        assert_approx_eq::assert_approx_eq!(block_out[4][0], 0.0, 0.0001);
        assert_approx_eq::assert_approx_eq!(block_out[5][0], 0.0, 0.0001);
        assert_approx_eq::assert_approx_eq!(block_out[6][0], 1.0, 0.0001);
        assert_approx_eq::assert_approx_eq!(block_out[7][0], 0.0, 0.0001);

        pchan.set_parameter(SynthParameterLabel::ChannelPosition, 2.0);
        block_out = pchan.process_block(block);

        assert_approx_eq::assert_approx_eq!(block_out[0][0], 0.0, 0.0001);
        assert_approx_eq::assert_approx_eq!(block_out[1][0], 0.0, 0.0001);
        assert_approx_eq::assert_approx_eq!(block_out[2][0], 1.0, 0.0001);
        assert_approx_eq::assert_approx_eq!(block_out[3][0], 0.0, 0.0001);
        assert_approx_eq::assert_approx_eq!(block_out[4][0], 0.0, 0.0001);
        assert_approx_eq::assert_approx_eq!(block_out[5][0], 0.0, 0.0001);
        assert_approx_eq::assert_approx_eq!(block_out[6][0], 0.0, 0.0001);
        assert_approx_eq::assert_approx_eq!(block_out[7][0], 0.0, 0.0001);

        pchan.set_parameter(SynthParameterLabel::ChannelPosition, 0.0);
        block_out = pchan.process_block(block);

        assert_approx_eq::assert_approx_eq!(block_out[0][0], 1.0, 0.0001);
        assert_approx_eq::assert_approx_eq!(block_out[1][0], 0.0, 0.0001);
        assert_approx_eq::assert_approx_eq!(block_out[2][0], 0.0, 0.0001);
        assert_approx_eq::assert_approx_eq!(block_out[3][0], 0.0, 0.0001);
        assert_approx_eq::assert_approx_eq!(block_out[4][0], 0.0, 0.0001);
        assert_approx_eq::assert_approx_eq!(block_out[5][0], 0.0, 0.0001);
        assert_approx_eq::assert_approx_eq!(block_out[6][0], 0.0, 0.0001);
        assert_approx_eq::assert_approx_eq!(block_out[7][0], 0.0, 0.0001);

        pchan.set_parameter(SynthParameterLabel::ChannelPosition, 8.0);
        block_out = pchan.process_block(block);

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
