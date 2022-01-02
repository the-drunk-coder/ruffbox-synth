use chfft::RFft1D;
use num_complex::*;

/**
 * A simple, non-partitioned block convolver.
 * Uses the Overlap-Save method (also called Overlap-Scrap)
 * for block convolution.
 */
pub struct BlockConvolver<const BUFSIZE: usize> {
    ir_freq_domain: Vec<Complex<f32>>,
    in_freq_domain: Vec<Complex<f32>>,
    fft: RFft1D<f32>,
    tmp_in: Vec<f32>,
    tmp_out: Vec<f32>,
    remainder: Vec<f32>,
    len: usize,
}

impl<const BUFSIZE: usize> std::clone::Clone for BlockConvolver<BUFSIZE> {
    fn clone(&self) -> Self {
        let fft = RFft1D::<f32>::new(self.len);

        BlockConvolver {
            ir_freq_domain: self.ir_freq_domain.clone(),
            in_freq_domain: self.in_freq_domain.clone(),
            fft,
            tmp_in: vec![0.0; 256],
            tmp_out: vec![0.0; 256],
            remainder: vec![0.0; 128],
            len: self.len,
        }
    }
}

impl<const BUFSIZE: usize> BlockConvolver<BUFSIZE> {
    // create block convolver from ir
    pub fn from_ir(ir: &[f32]) -> Self {
        // check if IR len == BUFSIZE ?

        let mut fft = RFft1D::<f32>::new(ir.len() * 2);

        // zero-pad impulse response (to match IR lenght)
        let mut ir_zeropad = vec![0.0; BUFSIZE * 2];
        ir_zeropad[..(BUFSIZE / 2)].copy_from_slice(&ir[..(BUFSIZE / 2)]);

        BlockConvolver {
            ir_freq_domain: fft.forward(&ir_zeropad),
            in_freq_domain: vec![Complex::new(0.0, 0.0); ir.len() * 2],
            fft,
            tmp_in: vec![0.0; BUFSIZE * 2],
            tmp_out: vec![0.0; BUFSIZE * 2],
            remainder: vec![0.0; BUFSIZE],
            len: ir.len() * 2,
        }
    }

    pub fn convolve(&mut self, input: [f32; BUFSIZE]) -> [f32; BUFSIZE] {
        // assemble input block from remainder part from previous block
        // (in this case, as filter length is always equal to blocksize,
        // the remainder is just the previous block)
        self.tmp_in[..BUFSIZE].copy_from_slice(&self.remainder[..BUFSIZE]);
        self.tmp_in[BUFSIZE..(2 * BUFSIZE) - 1].copy_from_slice(&input[..BUFSIZE]);

        // perform fft
        self.in_freq_domain = self.fft.forward(&self.tmp_in);

        // pointwise convolution
        for i in 0..self.in_freq_domain.len() {
            self.in_freq_domain[i] = self.ir_freq_domain[i] * self.in_freq_domain[i];
        }

        // ifft
        self.tmp_out = self.fft.backward(&self.in_freq_domain);

        // copy relevant part from ifft, scrap the rest
        let mut outarr = [0.0; BUFSIZE];
        self.remainder[..128].copy_from_slice(&input[..128]);
        outarr[..128].copy_from_slice(&self.tmp_out[BUFSIZE..(128 + BUFSIZE)]);

        // return result block ...
        outarr
    }
}

// TEST TEST TEST
#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_block_convolver_freq_domain_impulse_convolution() {
        // test convolution with impulse ...
        let mut ir = vec![0.0; 128];
        ir[0] = 1.0;

        let mut signal_in = [0.0; 128];

        let mut conv = BlockConvolver::<128>::from_ir(&ir);

        let mut dev_accum = 0.0;

        for b in 0..100 {
            for i in 0..128 {
                let pi_idx = ((b * 128 + i) as f32) * PI;
                signal_in[i] = ((220.0 / 44100.0) * pi_idx).sin();
                signal_in[i] += ((432.0 / 44100.0) * pi_idx).sin();
                signal_in[i] += ((648.0 / 44100.0) * pi_idx).sin();
            }
            let signal_out = conv.convolve(signal_in);
            for i in 0..128 {
                dev_accum += (signal_out[i] - signal_in[i]) * (signal_out[i] - signal_in[i]);
            }
        }

        assert_approx_eq::assert_approx_eq!(dev_accum / (100.0 * 128.0), 0.0, 0.00001);
    }
}
