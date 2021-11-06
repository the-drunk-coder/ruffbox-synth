use chfft::RFft1D;
use num_complex::*;

use crate::ruffbox::synth::convolver::precomputed_filter::*;

pub struct UniformPartitionedConvolution<const BUFSIZE: usize> {
    max_filter_length: usize,
    input_size: usize,
    num_sub_filters: usize,
    precomputed_sub_filters: Vec<PrecomputedFilter>,    
    frequency_delay_line: Vec<Vec<Complex<f32>>>,
    frequency_delay_line_idx: usize,
    output_accumulator: Vec<Complex<f32>>,
    fft: RFft1D<f32>,
    tmp_in: Vec<f32>,
    tmp_out: Vec<f32>,
    remainder: Vec<f32>,
}

/// UPOLS (Uniform Partitioned OverLap-Save), following Wefers, 2014
impl <const BUFSIZE: usize> UniformPartitionedConvolution<BUFSIZE> {

    /// prepare uniform partitioned convolution for specified filter length
    pub fn with_max_filter_length(mut max_filter_length: usize) -> UniformPartitionedConvolution<BUFSIZE> {

	// pad to next power of two:
	if !max_filter_length.is_power_of_two() {
	    max_filter_length = max_filter_length.next_power_of_two();
	}

	// calculate the number of needed sub-filters
	let num_sub_filters = max_filter_length / BUFSIZE;
	let mut precomputed_sub_filters = Vec::new();

	for _ in 0..num_sub_filters {
	    // sub-filter length is always the buffer size in this case
	    precomputed_sub_filters.push(PrecomputedFilter::with_max_filter_length(BUFSIZE));
	}

	let fft = RFft1D::<f32>::new(BUFSIZE * 2);
	
	UniformPartitionedConvolution {
	    max_filter_length,
	    input_size: BUFSIZE,
	    num_sub_filters,
	    precomputed_sub_filters,
	    frequency_delay_line: vec![vec![Complex::new(0.0, 0.0); BUFSIZE + 1]; num_sub_filters],
	    frequency_delay_line_idx: 0,
	    output_accumulator: vec![Complex::new(0.0, 0.0); BUFSIZE + 1],
	    fft,
	    tmp_in: vec![0.0; 2 * BUFSIZE],
            tmp_out: vec![0.0; 2 * BUFSIZE],
            remainder: vec![0.0; BUFSIZE],
	}
    }

    /// add a filter (impulse response) to this convolution
    /// the filter is passed by value so we can eventually pad it.
    /// if the filter is too long, it will be cut to size
    pub fn add(&mut self, mut filter: Vec<f32>) {
	// extend or truncate to size
	filter.resize(self.max_filter_length, 0.0);
	// fft is done in precomputed filter
	for i in 0..self.num_sub_filters {
	    self.precomputed_sub_filters[i].add(&filter[(i * self.input_size)..((i+1) * self.input_size)]);	    
	}	
    }

    /// add filter (impulse response) to this convolution, overwrite existing filter
    /// the filter is passed by value so we can eventually pad it. 
    /// if the filter is too long, it will be cut to size
    pub fn set(&mut self, mut filter: Vec<f32>) {
	// extend or truncate to size
	filter.resize(self.max_filter_length, 0.0);
	// fft is done in precomputed filter
	for i in 0..self.precomputed_sub_filters.len() {
	    self.precomputed_sub_filters[i].set(&filter[(i * self.input_size)..((i+1) * self.input_size)]);	    
	}
    }

    /// perform the convolution
    pub fn convolve(&mut self, input: [f32; BUFSIZE]) -> [f32; BUFSIZE] {
        // assemble input block from remainder part from previous block
        // (in this case, as filter length is always equal to blocksize,
        // the remainder is just the previous block)
        for i in 0..BUFSIZE {
            self.tmp_in[i] = self.remainder[i];
            self.tmp_in[i + BUFSIZE] = input[i];
        }

        // perform fft
	self.frequency_delay_line[self.frequency_delay_line_idx] = self.fft.forward(&self.tmp_in);

	let mut current_idx = self.frequency_delay_line_idx;

	// clear output accum
	for c in &mut self.output_accumulator {
	    c.re = 0.0;
	    c.im = 0.0;
	}
	
	// process the frequency delay line
	for f in 0..self.num_sub_filters {
	    self.precomputed_sub_filters[f].apply_add(&self.frequency_delay_line[current_idx], &mut self.output_accumulator);
	    if current_idx > 0 {
		current_idx -= 1;
	    } else {
		current_idx = self.num_sub_filters - 1;
	    }
	}

	self.frequency_delay_line_idx += 1;
	if self.frequency_delay_line_idx >= self.num_sub_filters {
	    self.frequency_delay_line_idx = 0;
	}

	self.tmp_out = self.fft.backward(&self.output_accumulator);
	
	// copy relevant part from ifft, scrap the rest
        let mut outarr = [0.0; BUFSIZE];
        for i in 0..self.input_size {
            self.remainder[i] = input[i];
            outarr[i] = self.tmp_out[i + BUFSIZE];
        }

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
    fn test_fdl_convolver_freq_domain_impulse_convolution_no_resize() {
        // test convolution with impulse ...
        let mut ir = vec![0.0; 512];
        ir[0] = 1.0;

        let mut signal_in = [0.0; 128];

        let mut conv = UniformPartitionedConvolution::<128>::with_max_filter_length(512);
	conv.set(ir);

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

