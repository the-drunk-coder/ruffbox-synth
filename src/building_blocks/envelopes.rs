pub mod source_env;

pub mod exp_perc_envelope;
pub mod linear_asr_envelope;
pub mod multi_point_envelope;

pub use crate::building_blocks::envelopes::exp_perc_envelope::ExpPercEnvelope;
pub use crate::building_blocks::envelopes::linear_asr_envelope::LinearASREnvelope;
pub use crate::building_blocks::envelopes::multi_point_envelope::MultiPointEffectEnvelope;

// TEST TEST TEST
#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use crate::building_blocks::{
        EnvelopeSegmentInfo, EnvelopeSegmentType, MonoEffect, SynthParameterLabel,
        SynthParameterValue, ValOp,
    };

    /// test the general workings of the ASREnvelope
    #[test]
    fn test_asr_envelope() {
        let test_block: [f32; 128] = [1.0; 128];

        // half a block attack, one block sustain, half a block release ... 2 blocks total .
        let mut env = LinearASREnvelope::<128>::new(0.5, 0.0014512, 0.0029024, 0.0014512, 44100.0);

        let out_1: [f32; 128] = env.process_block(test_block, 0, &Vec::new());
        let out_2: [f32; 128] = env.process_block(test_block, 0, &Vec::new());

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
        let mut env = LinearASREnvelope::<128>::new(0.0, 0.0, 0.0, 0.0, 44100.0);

        // use paramter setter to set parameters ...
        env.set_parameter(
            SynthParameterLabel::Attack,
            &SynthParameterValue::ScalarF32(0.0014512),
        );
        env.set_parameter(
            SynthParameterLabel::Sustain,
            &SynthParameterValue::ScalarF32(0.0029024),
        );
        env.set_parameter(
            SynthParameterLabel::Release,
            &SynthParameterValue::ScalarF32(0.0014512),
        );
        env.set_parameter(
            SynthParameterLabel::EnvelopeLevel,
            &SynthParameterValue::ScalarF32(0.5),
        );

        let out_1: [f32; 128] = env.process_block(test_block, 0, &Vec::new());
        let out_2: [f32; 128] = env.process_block(test_block, 0, &Vec::new());

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
        let mut env_at_start = LinearASREnvelope::<128>::new(0.0, 0.0, 0.0, 0.0, 44100.0);
        // let this one start somewhere in the block
        let mut env_with_offset = LinearASREnvelope::<128>::new(0.0, 0.0, 0.0, 0.0, 44100.0);

        // use paramter setter to set parameters ...
        println!("Set parameters for env at start:");
        env_at_start.set_parameter(
            SynthParameterLabel::EnvelopeLevel,
            &SynthParameterValue::ScalarF32(1.0),
        );
        env_at_start.set_parameter(
            SynthParameterLabel::Attack,
            &SynthParameterValue::ScalarF32(0.001),
        );
        env_at_start.set_parameter(
            SynthParameterLabel::Sustain,
            &SynthParameterValue::ScalarF32(0.019),
        );
        env_at_start.set_parameter(
            SynthParameterLabel::Release,
            &SynthParameterValue::ScalarF32(0.07),
        );

        println!("\nSet parameters for env with offset:\n");
        env_with_offset.set_parameter(
            SynthParameterLabel::EnvelopeLevel,
            &SynthParameterValue::ScalarF32(1.0),
        );
        env_with_offset.set_parameter(
            SynthParameterLabel::Attack,
            &SynthParameterValue::ScalarF32(0.001),
        );
        env_with_offset.set_parameter(
            SynthParameterLabel::Sustain,
            &SynthParameterValue::ScalarF32(0.019),
        );
        env_with_offset.set_parameter(
            SynthParameterLabel::Release,
            &SynthParameterValue::ScalarF32(0.07),
        );

        let mut out_start = env_at_start.process_block(test_block, 0, &Vec::new());
        let mut out_offset = env_with_offset.process_block(test_block, 60, &Vec::new());

        // calculate 34 blocks
        for _ in 0..34 {
            for i in 0..68 {
                //print!("{} {} - ", out_start[i], out_offset[i + 60]);
                assert_approx_eq::assert_approx_eq!(out_start[i], out_offset[i + 60], 0.00001);
            }
            //println!{" block {}.1 done \n", i};

            out_offset = env_with_offset.process_block(test_block, 0, &Vec::new());

            for i in 68..128 {
                //print!("{} {} - ", out_start[i], out_offset[i - 68]);
                assert_approx_eq::assert_approx_eq!(out_start[i], out_offset[i - 68], 0.00001);
            }

            //println!{" block {}.2 done \n", i};
            out_start = env_at_start.process_block(test_block, 0, &Vec::new());
        }
    }

    #[test]
    fn perc_exp_smoke_test() {
        let mut exp_env = ExpPercEnvelope::<128>::new(1.0, 0.05, 0.0, 1.0, 16000.0);
        let test_block: [f32; 128] = [1.0; 128];
        let mut out = Vec::new();
        for _ in 0..132 {
            let env_out = exp_env.process_block(test_block, 0, &Vec::new());
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

    #[test]
    fn test_multi_point_effect_env() {
        let segments = vec![
            EnvelopeSegmentInfo {
                from: 0.0,
                to: 0.7,
                time: 0.01,
                segment_type: EnvelopeSegmentType::Log,
            },
            EnvelopeSegmentInfo {
                from: 0.7,
                to: 0.7,
                time: 0.48,
                segment_type: EnvelopeSegmentType::Constant,
            },
            EnvelopeSegmentInfo {
                from: 0.7,
                to: 0.0,
                time: 0.01,
                segment_type: EnvelopeSegmentType::Log,
            },
        ];

        let mut mpenv = MultiPointEffectEnvelope::<512>::empty(44100.0);
        mpenv.set_parameter(
            SynthParameterLabel::Envelope,
            &SynthParameterValue::MultiPointEnvelope(
                segments,
                false,
                crate::building_blocks::ValOp::Replace,
            ),
        );

        let in_block = [1.0; 512];
        let num_blocks = (1.3 * 44100.0 / 512.0) as usize;

        for _ in 0..num_blocks {
            let _ = mpenv.process_block(in_block, 0, &Vec::new());
            //let block = mpenv.get_next_block(0, &Vec::new());
            //for i in 0..512 {
            //    let a = out_block[i];
            //    debug_plotter::plot!(a where caption = "MultiPointTest");
            //}
        }
    }

    #[test]
    fn test_compare_lin_asr_multipoint() {
        let test_block: [f32; 512] = [1.0; 512];

        // half a block attack, one block sustain, half a block release ... 2 blocks total .
        let mut env1 = LinearASREnvelope::<512>::new(1.0, 0.0, 1.0, 0.0, 44100.0);
        let mut env2 = multi_point_envelope::MultiPointEffectEnvelope::<512>::empty(44100.0);

        env2.set_parameter(
            SynthParameterLabel::Envelope,
            &SynthParameterValue::MultiPointEnvelope(
                vec![
                    EnvelopeSegmentInfo {
                        from: 0.0,
                        to: 1.0,
                        time: 0.000025,
                        segment_type: EnvelopeSegmentType::Lin,
                    },
                    EnvelopeSegmentInfo {
                        from: 1.0,
                        to: 1.0,
                        time: 1.0,
                        segment_type: EnvelopeSegmentType::Constant,
                    },
                    EnvelopeSegmentInfo {
                        from: 1.0,
                        to: 0.0,
                        time: 0.0,
                        segment_type: EnvelopeSegmentType::Lin,
                    },
                ],
                false,
                ValOp::Replace,
            ),
        );

        let num_blocks = 44100 / 512 + 1;

        for i in 0..num_blocks {
            let out1 = env1.process_block(test_block, 0, &Vec::new());
            let out2 = env2.process_block(test_block, 0, &Vec::new());

            if i == num_blocks - 1 {
                for i in 60..80 {
                    let a = out1[i];
                    let b = out2[i];
                    debug_plotter::plot!(a, b where caption = "MultiPointCompare");
                }
            }
        }
    }
}
