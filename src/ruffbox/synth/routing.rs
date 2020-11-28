use crate::ruffbox::synth::SynthParameter;
use std::f32::consts::PI;

pub struct PanChan<const BUFSIZE:usize, const NCHAN:usize> {
    levels: [f32; NCHAN],
}

impl <const BUFSIZE:usize, const NCHAN:usize> PanChan<BUFSIZE, NCHAN> {
    pub fn new() -> Self {
	let mut lvls = [0.0; NCHAN];
	lvls[0] = 1.0;
        // always start on first channel
        PanChan {            
            levels: lvls,            
        }
    }
    
    /// some parameter limits might be nice ... 
    pub fn set_parameter(&mut self, par: SynthParameter, value: f32) {
        match par {
            SynthParameter::StereoPosition => {
		let mut lvls = [0.0; NCHAN];
		
		let lower = value.floor() % (NCHAN as f32);
		let upper = value.ceil() % (NCHAN as f32);
                let angle_rad = -1.0 * (value) * PI * 0.25;
                let angle_cos = angle_rad.cos();
                let angle_sin = angle_rad.sin();
                let sqrt_two_half = (2.0 as f32).sqrt() / 2.0;
                lvls[lower as usize] = sqrt_two_half * (angle_cos + angle_sin);
                lvls[upper as usize] = sqrt_two_half * (angle_cos - angle_sin);
		self.levels = lvls;
            },
            _ => (),
        };
    }
    /// pan mono to stereo
    pub fn process_block(&mut self, block: [f32; BUFSIZE]) -> [[f32; BUFSIZE]; NCHAN] {
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
    fn pann_test_basic_pan() {
        let mut bal2 = PanChan::<128,2>::new();

        let mut block = [0.0; 128];
        block[0] = 1.0;

	bal2.set_parameter(SynthParameter::StereoPosition, 0.5);
	
        let block_out = bal2.process_block(block);

        assert_approx_eq::assert_approx_eq!(block_out[0][0], 0.707, 0.001);
        assert_approx_eq::assert_approx_eq!(block_out[1][0], 0.707, 0.001);
    }

    #[test]
    fn pann_test_left_pan() {
        let mut bal2 = PanChan::<128,2>::new();

        bal2.set_parameter(SynthParameter::StereoPosition, 0.0);
        
        let mut block = [0.0; 128];
        block[0] = 1.0;
        
        let block_out = bal2.process_block(block);

        assert_approx_eq::assert_approx_eq!(block_out[0][0], 1.0, 0.0001);
        assert_approx_eq::assert_approx_eq!(block_out[1][0], 0.0, 0.0001);
    }

    #[test]
    fn pann_test_right_pan() {
        let mut bal2 = PanChan::<128,2>::new();

        bal2.set_parameter(SynthParameter::StereoPosition, 1.0);
        
        let mut block = [0.0; 128];
        block[0] = 1.0;
        
        let block_out = bal2.process_block(block);

        assert_approx_eq::assert_approx_eq!(block_out[0][0], 0.0, 0.0001);
        assert_approx_eq::assert_approx_eq!(block_out[1][0], 1.0, 0.0001);
    }
}
