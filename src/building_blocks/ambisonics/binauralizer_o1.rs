use crate::building_blocks::convolver::block_convolver::BlockConvolver;
use rubato::{FftFixedIn, Resampler};

// 4x128 points @ 44100kHz, raw f32 ...
const DEFAULT_FILTER: &'static [u8] = include_bytes!("../../../binaural_filter/default.raw");

/**
 * a simple first-order convolution binauralizer
 */
pub struct BinauralizerO1<const BUFSIZE: usize> {
    left: Vec<BlockConvolver<BUFSIZE>>,
    right: Vec<BlockConvolver<BUFSIZE>>,
}

impl<const BUFSIZE: usize> BinauralizerO1<BUFSIZE> {
    pub fn default_filter(samplerate: f32) -> Self {
        let ir: Vec<f32> = DEFAULT_FILTER
            .chunks(4)
            .map(|b| (f32::from_le_bytes(b.try_into().unwrap()) as f32))
            .collect();

        debug_assert!(ir.len() == 1024);

        let mut ir_proc: Vec<(Vec<f32>, Vec<f32>)> = vec![
            (ir[0..128].to_vec(), ir[512..640].to_vec()),
            (ir[128..256].to_vec(), ir[640..768].to_vec()),
            (ir[256..384].to_vec(), ir[768..896].to_vec()),
            (ir[384..512].to_vec(), ir[896..1024].to_vec()),
        ];

        if samplerate != 44100.0 {
            for i in 0..4 {
                let (l, r) = ir_proc.get(i).unwrap();
                let mut resampler_l = FftFixedIn::<f32>::new(44100, samplerate as usize, 128, 1, 1);
                let mut resampler_r = FftFixedIn::<f32>::new(44100, samplerate as usize, 128, 1, 1);
                let l_resampled = resampler_l.process(&vec![l]).unwrap();
                let r_resampled = resampler_r.process(&vec![r]).unwrap();
                ir_proc[i] = (l_resampled[0].clone(), r_resampled[0].clone());
            }
        }
        // in case we have bufsizes larger than 128 (most of the time I guess) ...
        let zero_pad = BUFSIZE - ir_proc[0].0.len();
        if zero_pad > 0 {
            for (l, r) in ir_proc.iter_mut() {
                l.append(&mut vec![0.0; zero_pad]);
                r.append(&mut vec![0.0; zero_pad]);
            }
        }

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
