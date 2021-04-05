use crate::ruffbox::synth::convolver::block_convolver::BlockConvolver;
/**
 * a simple first-order convolution binauralizer
 */
pub struct BinauralizerO1<const BUFSIZE: usize> {
    left: Vec<BlockConvolver<BUFSIZE>>,
    right: Vec<BlockConvolver<BUFSIZE>>,
}

impl<const BUFSIZE: usize> BinauralizerO1<BUFSIZE> {
    // initialize with unit IRs
    pub fn from_ir(ir: Vec<(Vec<f32>, Vec<f32>)>) -> Self {
        let mut left = Vec::new();
        let mut right = Vec::new();

        for i in 0..4 {
            left.push(BlockConvolver::<BUFSIZE>::from_ir(&ir[i].0));
            right.push(BlockConvolver::<BUFSIZE>::from_ir(&ir[i].1))
        }

        BinauralizerO1 { left, right }
    }

    pub fn binauralize(&mut self, input: &[[f32; BUFSIZE]; 4]) -> [[f32; BUFSIZE]; 2] {
        let mut bin_block = [[0.0; BUFSIZE]; 2];

        for ach in 0..4 {
            let lch = self.left[ach].convolve(input[ach]);
            let rch = self.right[ach].convolve(input[ach]);
            for fr in 0..BUFSIZE {
                bin_block[0][fr] += lch[fr];
                bin_block[1][fr] += rch[fr];
            }
        }

        bin_block
    }
}
