use crate::building_blocks::convolver::uniform_partitioned_convolution::UniformPartitionedConvolution;
use crate::MultichannelReverb;
use crate::{SynthParameterLabel, SynthParameterValue};

pub struct MultichannelConvolutionReverb<const BUFSIZE: usize, const NCHAN: usize> {
    channel_convolvers: Vec<UniformPartitionedConvolution<BUFSIZE>>,
}

impl<const BUFSIZE: usize, const NCHAN: usize> MultichannelConvolutionReverb<BUFSIZE, NCHAN> {
    pub fn with_ir(ir: &[f32]) -> MultichannelConvolutionReverb<BUFSIZE, NCHAN> {
        let mut channel_convolvers = Vec::new();
        for _ in 0..NCHAN {
            channel_convolvers.push(UniformPartitionedConvolution::with_ir(ir.to_vec()));
        }
        MultichannelConvolutionReverb { channel_convolvers }
    }
}

impl<const BUFSIZE: usize, const NCHAN: usize> MultichannelReverb<BUFSIZE, NCHAN>
    for MultichannelConvolutionReverb<BUFSIZE, NCHAN>
{
    fn set_parameter(&mut self, _par: SynthParameterLabel, _val: &SynthParameterValue) {
        // nothing to do here ...
    }

    /**
     * Main processing routine.
     * Takes a mono block, as this would be downmixed anyway.
     */
    fn process(&mut self, block: [[f32; BUFSIZE]; NCHAN]) -> [[f32; BUFSIZE]; NCHAN] {
        let mut out_buf = [[0.0; BUFSIZE]; NCHAN];

        for c in 0..NCHAN {
            out_buf[c] = self.channel_convolvers[c].convolve(block[c]);
        }

        out_buf
    }
}
