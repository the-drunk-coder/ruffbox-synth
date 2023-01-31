use crate::building_blocks::convolver::block_convolver::BlockConvolver;

const DEFAULT_FILTER : &'static [u8] = include_bytes!("../../../binaural_filter/default.raw");

/**
 * a simple first-order convolution binauralizer
 */
pub struct BinauralizerO1<const BUFSIZE: usize> {
    left: Vec<BlockConvolver<BUFSIZE>>,
    right: Vec<BlockConvolver<BUFSIZE>>,
}

impl<const BUFSIZE: usize> BinauralizerO1<BUFSIZE> {

    pub fn default_filter() -> Self {
	 let ir: Vec<f32> = DEFAULT_FILTER
            .chunks(4)
            .map(|b| (f32::from_le_bytes(b.try_into().unwrap()) as f32))
            .collect();

	debug_assert!(ir.len() == 1024);
	
	let ir_proc :Vec<(Vec<f32>, Vec<f32>)> = vec![
	    (ir[0..128].to_vec(), ir[512..640].to_vec()),
	    (ir[128..256].to_vec(), ir[640..768].to_vec()),
	    (ir[256..384].to_vec(), ir[768..896].to_vec()),
	    (ir[384..512].to_vec(), ir[896..1024].to_vec()),
	];
	
	BinauralizerO1::from_ir(ir_proc)
    }

    // initialize with unit IRs
    pub fn from_ir(ir: Vec<(Vec<f32>, Vec<f32>)>) -> Self {
        let mut left = Vec::new();
        let mut right = Vec::new();

        for i in ir.iter().take(4) {
            left.push(BlockConvolver::<BUFSIZE>::from_ir(&i.0));
            right.push(BlockConvolver::<BUFSIZE>::from_ir(&i.1))
        }

        BinauralizerO1 { left, right }
    }

    pub fn binauralize(&mut self, input: [[f32; BUFSIZE]; 4]) -> [[f32; BUFSIZE]; 2] {
        let mut bin_block = [[0.0; BUFSIZE]; 2];

        for (ach, i) in input.iter().enumerate().take(4) {
            let lch = self.left[ach].convolve(*i);
            let rch = self.right[ach].convolve(*i);
            for fr in 0..BUFSIZE {
                bin_block[0][fr] += lch[fr];
                bin_block[1][fr] += rch[fr];
            }
        }

        bin_block
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    #[test]
    fn test_load() {
	let bin = BinauralizerO1::<128>::default_filter();
    }
}
