mod bal_chan;
mod pan_chan; // pan mono // balance stereo

pub use bal_chan::BalChan;
pub use pan_chan::PanChan;

// TEST TEST TEST
#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use crate::building_blocks::{
        SynthParameterLabel, SynthParameterValue,
    };
    #[test]
    fn panchan_test_basic_pan() {
        let mut pchan = PanChan::<128, 2>::new();

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
        let mut pchan = PanChan::<128, 2>::new();

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
        let mut pchan = PanChan::<128, 2>::new();

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
        let mut pchan = PanChan::<128, 8>::new();

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
