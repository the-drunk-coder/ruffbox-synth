/// A collection of oscillators, some of which are modeled
/// after scsynth, csound, etc ...
pub mod fm_saw;
pub mod fm_square;
pub mod fm_tri;
pub mod lf_cub;
pub mod lf_rsaw;
pub mod lf_saw;
pub mod lf_square;
pub mod lf_tri;
pub mod sine_osc;
pub mod wavematrix;
pub mod wavetable;
pub mod wt_saw;

pub use crate::building_blocks::oscillators::lf_cub::LFCub;
pub use crate::building_blocks::oscillators::lf_rsaw::LFRSaw;
pub use crate::building_blocks::oscillators::lf_saw::LFSaw;
pub use crate::building_blocks::oscillators::lf_square::LFSquare;
pub use crate::building_blocks::oscillators::lf_tri::LFTri;
pub use crate::building_blocks::oscillators::sine_osc::SineOsc;
pub use crate::building_blocks::oscillators::wavematrix::Wavematrix;
pub use crate::building_blocks::oscillators::wavetable::Wavetable;

pub use crate::building_blocks::oscillators::fm_saw::FMSaw;
pub use crate::building_blocks::oscillators::fm_square::FMSquare;
pub use crate::building_blocks::oscillators::fm_tri::FMTri;
pub use crate::building_blocks::oscillators::wt_saw::WTSaw;

// TEST TEST TEST
#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use crate::building_blocks::MonoSource;

    use std::f32::consts::PI;

    #[test]
    fn sine_osc_test_at_block_start() {
        let mut osc = SineOsc::<128>::new(440.0, 1.0, 44100.0);

        let out_1 = osc.get_next_block(0, &Vec::new());
        let mut comp_1 = [0.0; 128];

        for i in 0..128 {
            comp_1[i] = (2.0 * PI * 440.0 * (i as f32 * (1.0 / 44100.0))).sin()
        }

        // the new sine osc seems to be a bit less precise ....
        for i in 0usize..128usize {
            //let b = out_1[i];
            //let c = comp_1[i];

            //debug_plotter::plot!(b, c where caption = "BlockPlot");
            assert_approx_eq::assert_approx_eq!(out_1[i], comp_1[i], 0.008);
        }
    }

    /*
    #[test]
    fn sine_osc_rel_phase_offset() {
        let mut osc = SineOsc::<128>::new(440.0, 1.0, 44100.0);

    osc.set_parameter(SynthParameterLabel::OscillatorPhaseRelative,
              &SynthParameterValue::ScalarF32(0.5));

        let out_1 = osc.get_next_block(0, &Vec::new());
        let mut comp_1 = [0.0; 128];

        for i in 0..128 {
            comp_1[i] = (2.0 * PI * 440.0 * (i as f32 * (1.0 / 44100.0))).cos()
        }

        // the new sine osc seems to be a bit less precise ....
        for i in 0usize..128usize {
            let b = out_1[i];
            let c = comp_1[i];

            debug_plotter::plot!(b, c  where caption = "BlockPlotRelPhase");
            //assert_approx_eq::assert_approx_eq!(out_1[i], comp_1[i], 0.008);
        }
    }

    #[test]
    fn sine_osc_abs_phase_offset() {
        let mut osc = SineOsc::<128>::new(440.0, 200.0, 44100.0);

    osc.set_parameter(SynthParameterLabel::OscillatorPhaseEffective,
              &SynthParameterValue::ScalarF32(100.0));

        let out_1 = osc.get_next_block(0, &Vec::new());
        let mut comp_1 = [0.0; 128];

        for i in 0..128 {
            comp_1[i] = (2.0 * PI * 440.0 * (i as f32 * (1.0 / 44100.0))).sin() * 200.0
        }

        // the new sine osc seems to be a bit less precise ....
        for i in 0usize..128usize {
            let b = out_1[i];
            let c = comp_1[i];

            debug_plotter::plot!(b, c  where caption = "BlockPlotAbsPhase");
            //assert_approx_eq::assert_approx_eq!(out_1[i], comp_1[i], 0.008);
        }
}*/
    #[test]
    fn plot_tri() {
        let mut osc = FMTri::<128>::new(440.0, 200.0, 44100.0);
	
	
	for _ in 0..20 {
	    let out_1 = osc.get_next_block(0, &Vec::new());
            
	    
            // the new sine osc seems to be a bit less precise ....
            for i in 0usize..128usize {
		let b = out_1[i];
				
		debug_plotter::plot!(b  where caption = "PlotFmTri");		
            }
	}
    }

    #[test]
    fn sine_osc_test_start_in_block() {
        let mut osc = SineOsc::<128>::new(440.0, 1.0, 44100.0);

        let start_time: f32 = 0.001;

        let sample_offset = (44100.0 * start_time).round() as usize;

        let out_1 = osc.get_next_block(sample_offset, &Vec::new());

        let mut comp_1 = [0.0; 128];

        for i in sample_offset..128 {
            comp_1[i] = (2.0 * PI * 440.0 * ((i - sample_offset) as f32 * (1.0 / 44100.0))).sin()
        }

        for i in 0..128 {
            //println!("{} {} {}; ", i, out_1[i], comp_1[i]);
            assert_approx_eq::assert_approx_eq!(out_1[i], comp_1[i], 0.008);
        }
    }

    #[test]
    fn sine_osc_test_multiple_blocks() {
        let mut osc = SineOsc::<128>::new(440.0, 1.0, 44100.0);

        let out_1 = osc.get_next_block(0, &Vec::new());
        let out_2 = osc.get_next_block(0, &Vec::new());
        let out_3 = osc.get_next_block(0, &Vec::new());
        let out_4 = osc.get_next_block(0, &Vec::new());
        let out_5 = osc.get_next_block(0, &Vec::new());
        let out_6 = osc.get_next_block(0, &Vec::new());

        let mut comp_1 = [0.0; 128];
        let mut comp_2 = [0.0; 128];
        let mut comp_3 = [0.0; 128];
        let mut comp_4 = [0.0; 128];
        let mut comp_5 = [0.0; 128];
        let mut comp_6 = [0.0; 128];

        for i in 0..128 {
            comp_1[i] = (2.0 * PI * 440.0 * (i as f32 * (1.0 / 44100.0))).sin()
        }

        for i in 128..256 {
            comp_2[i - 128] = (2.0 * PI * 440.0 * (i as f32 * (1.0 / 44100.0))).sin()
        }

        for i in 256..384 {
            comp_3[i - 256] = (2.0 * PI * 440.0 * (i as f32 * (1.0 / 44100.0))).sin()
        }

        for i in 384..512 {
            comp_4[i - 384] = (2.0 * PI * 440.0 * (i as f32 * (1.0 / 44100.0))).sin()
        }

        for i in 512..640 {
            comp_5[i - 512] = (2.0 * PI * 440.0 * (i as f32 * (1.0 / 44100.0))).sin()
        }

        for i in 640..768 {
            comp_6[i - 640] = (2.0 * PI * 440.0 * (i as f32 * (1.0 / 44100.0))).sin()
        }

        for i in 0..128 {
            // this isn't very precise ???
            //println!("{} {} {}; ", i, out_1[i], comp_1[i]);
            assert_approx_eq::assert_approx_eq!(out_1[i], comp_1[i], 0.008);
        }
        for i in 0..128 {
            // this isn't very precise ???
            //println!("{} {} {}; ", i, out_2[i], comp_2[i]);
            assert_approx_eq::assert_approx_eq!(out_2[i], comp_2[i], 0.008);
        }
        for i in 0..128 {
            // this isn't very precise ???
            //println!("{} {} {}; ", i, out_3[i], comp_3[i]);
            assert_approx_eq::assert_approx_eq!(out_3[i], comp_3[i], 0.008);
        }
        for i in 0..128 {
            // this isn't very precise ???
            //println!("{} {} {}; ", i, out_1[i], comp_1[i]);
            assert_approx_eq::assert_approx_eq!(out_4[i], comp_4[i], 0.008);
        }
        for i in 0..128 {
            // this isn't very precise ???
            //println!("{} {} {}; ", i, out_2[i], comp_2[i]);
            assert_approx_eq::assert_approx_eq!(out_5[i], comp_5[i], 0.008);
        }
        for i in 0..128 {
            // this isn't very precise ???
            //println!("{} {} {}; ", i, out_3[i], comp_3[i]);
            assert_approx_eq::assert_approx_eq!(out_6[i], comp_6[i], 0.008);
        }
    }
}
