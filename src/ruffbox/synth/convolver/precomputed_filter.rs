use chfft::RFft1D;
use num_complex::*;

pub struct PrecomputedFilter {
    max_filter_length: usize,
    fft: RFft1D<f32>,
    spectrum: Vec<Complex<f32>>,
}

impl PrecomputedFilter {
    /// prepare pre-computed filter for specified filter length
    pub fn with_max_filter_length(mut max_filter_length: usize) -> PrecomputedFilter {
        // pad to next power of two:
        if !max_filter_length.is_power_of_two() {
            max_filter_length = max_filter_length.next_power_of_two();
        }

        let fft = RFft1D::<f32>::new(max_filter_length * 2);
        let spectrum = vec![Complex::new(0.0, 0.0); max_filter_length];
        PrecomputedFilter {
            max_filter_length,
            fft,
            spectrum,
        }
    }

    /// add a filter (impulse response) to this pre-computed filter
    pub fn add(&mut self, filter: &[f32]) {
        let mut workbuf = filter.to_vec();
        workbuf.resize(self.max_filter_length * 2, 0.0);
        let filter_freq_domain = self.fft.forward(&workbuf);
        for i in 0..self.spectrum.len() {
            self.spectrum[i] += filter_freq_domain[i];
        }
    }

    /// set the filter, overwrite existing fiter
    pub fn set(&mut self, filter: &[f32]) {
        let mut workbuf = filter.to_vec();
        workbuf.resize(self.max_filter_length * 2, 0.0);
        self.spectrum = self.fft.forward(&workbuf);
    }

    /// clear this filter
    pub fn clear(&mut self) {
        self.spectrum = vec![Complex::new(0.0, 0.0); self.max_filter_length];
    }

    /// apply this pre-computed filter to input, overwrite output
    pub fn apply(&mut self, input: &[Complex<f32>], output: &mut [Complex<f32>]) {
        for i in 0..input.len() {
            output[i] = self.spectrum[i] * input[i];
        }
    }

    /// apply this pre-computed filter to input, add to output
    pub fn apply_add(&mut self, input: &[Complex<f32>], output: &mut [Complex<f32>]) {
        for i in 0..input.len() {
            output[i] += self.spectrum[i] * input[i];
        }
    }
}
