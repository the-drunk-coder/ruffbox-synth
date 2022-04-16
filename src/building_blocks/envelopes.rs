pub mod exp_perc_envelope;
pub mod linear_asr_envelope;

pub use crate::building_blocks::envelopes::exp_perc_envelope::ExpPercEnvelope;
pub use crate::building_blocks::envelopes::linear_asr_envelope::LinearASREnvelope;

// TEST TEST TEST
#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    /// test the general workings of the ASREnvelope
    #[test]
    fn test_asr_envelope() {
        let test_block: [f32; 128] = [1.0; 128];

        // half a block attack, one block sustain, half a block release ... 2 blocks total .
        let mut env = ASREnvelope::<128>::new(0.5, 0.0014512, 0.0029024, 0.0014512, 44100.0);

        let out_1: [f32; 128] = env.process_block(test_block, 0);
        let out_2: [f32; 128] = env.process_block(test_block, 0);

        // comparison
        let mut comp_block_1: [f32; 128] = [0.0; 128];
        let mut comp_block_2: [f32; 128] = [0.0; 128];

        let mut gain = 0.0;
        let gain_inc_dec = 0.5 / 64.0;

        // fill comp blocks
        for i in 0..64 {
            comp_block_1[i] = test_block[i] * gain;
            gain += gain_inc_dec;
        }

        for i in 0..64 {
            comp_block_1[64 + i] = test_block[64 + i] * gain;
        }

        for i in 0..64 {
            comp_block_2[i] = test_block[i] * gain;
        }

        for i in 0..64 {
            gain -= gain_inc_dec;
            comp_block_2[64 + i] = test_block[64 + i] * gain;
        }

        for i in 0..128 {
            //println!("{} {}", out_1[i], comp_block_1[i]);
            assert_approx_eq::assert_approx_eq!(out_1[i], comp_block_1[i], 0.00001);
        }

        for i in 0..128 {
            //println!("{} {}", out_2[i], comp_block_2[i]);
            assert_approx_eq::assert_approx_eq!(out_2[i], comp_block_2[i], 0.00001);
        }
    }

    /// test the parameter setter of the envelope
    #[test]
    fn test_asr_envelope_set_params() {
        let test_block: [f32; 128] = [1.0; 128];

        // half a block attack, one block sustain, half a block release ... 2 blocks total .
        let mut env = ASREnvelope::<128>::new(0.0, 0.0, 0.0, 0.0, 44100.0);

        // use paramter setter to set parameters ...
        env.set_parameter(SynthParameterLabel::Attack, 0.0014512);
        env.set_parameter(SynthParameterLabel::Sustain, 0.0029024);
        env.set_parameter(SynthParameterLabel::Release, 0.0014512);
        env.set_parameter(SynthParameterLabel::Level, 0.5);

        let out_1: [f32; 128] = env.process_block(test_block, 0);
        let out_2: [f32; 128] = env.process_block(test_block, 0);

        // comparison
        let mut comp_block_1: [f32; 128] = [0.0; 128];
        let mut comp_block_2: [f32; 128] = [0.0; 128];

        let mut gain = 0.0;
        let gain_inc_dec = 0.5 / 64.0;

        // fill comp blocks
        for i in 0..64 {
            comp_block_1[i] = test_block[i] * gain;
            gain += gain_inc_dec;
        }

        for i in 0..64 {
            comp_block_1[64 + i] = test_block[64 + i] * gain;
        }

        for i in 0..64 {
            comp_block_2[i] = test_block[i] * gain;
        }

        for i in 0..64 {
            gain -= gain_inc_dec;
            comp_block_2[64 + i] = test_block[64 + i] * gain;
        }

        for i in 0..128 {
            //println!("{} {}", out_1[i], comp_block_1[i]);
            assert_approx_eq::assert_approx_eq!(out_1[i], comp_block_1[i], 0.00001);
        }

        for i in 0..128 {
            //println!("{} {}", out_2[i], comp_block_2[i]);
            assert_approx_eq::assert_approx_eq!(out_2[i], comp_block_2[i], 0.00001);
        }
    }

    #[test]
    fn test_asr_envelope_short_intervals_with_offset() {
        let test_block: [f32; 128] = [1.0; 128];

        // let this one start at the beginning of a block
        let mut env_at_start = ASREnvelope::<128>::new(0.0, 0.0, 0.0, 0.0, 44100.0);
        // let this one start somewhere in the block
        let mut env_with_offset = ASREnvelope::<128>::new(0.0, 0.0, 0.0, 0.0, 44100.0);

        // use paramter setter to set parameters ...
        println!("Set parameters for env at start:");
        env_at_start.set_parameter(SynthParameterLabel::Level, 1.0);
        env_at_start.set_parameter(SynthParameterLabel::Attack, 0.001);
        env_at_start.set_parameter(SynthParameterLabel::Sustain, 0.019);
        env_at_start.set_parameter(SynthParameterLabel::Release, 0.07);

        println!("\nSet parameters for env with offset:\n");
        env_with_offset.set_parameter(SynthParameterLabel::Level, 1.0);
        env_with_offset.set_parameter(SynthParameterLabel::Attack, 0.001);
        env_with_offset.set_parameter(SynthParameterLabel::Sustain, 0.019);
        env_with_offset.set_parameter(SynthParameterLabel::Release, 0.07);

        let mut out_start = env_at_start.process_block(test_block, 0);
        let mut out_offset = env_with_offset.process_block(test_block, 60);

        // calculate 34 blocks
        for _ in 0..34 {
            for i in 0..68 {
                //print!("{} {} - ", out_start[i], out_offset[i + 60]);
                assert_approx_eq::assert_approx_eq!(out_start[i], out_offset[i + 60], 0.00001);
            }
            //println!{" block {}.1 done \n", i};

            out_offset = env_with_offset.process_block(test_block, 0);

            for i in 68..128 {
                //print!("{} {} - ", out_start[i], out_offset[i - 68]);
                assert_approx_eq::assert_approx_eq!(out_start[i], out_offset[i - 68], 0.00001);
            }

            //println!{" block {}.2 done \n", i};
            out_start = env_at_start.process_block(test_block, 0);
        }
    }

    #[test]
    fn perc_exp_smoke_test() {
        let mut exp_env = ExpPercEnvelope::<128>::new(1.0, 0.05, 0.0, 1.0, 16000.0);
        let test_block: [f32; 128] = [1.0; 128];
        let mut out = Vec::new();
        for _ in 0..132 {
            let env_out = exp_env.process_block(test_block, 0);
            out.extend_from_slice(&env_out);
        }

        assert_approx_eq::assert_approx_eq!(out[0], 0.0, 0.00001);
        assert_approx_eq::assert_approx_eq!(out[800], 1.0, 0.00001);
        assert_approx_eq::assert_approx_eq!(out[16800], 0.0, 0.00001);

        for sample in out.iter() {
            assert!(*sample >= 0.0);
            assert!(*sample <= 1.0);
        }
    }
}
