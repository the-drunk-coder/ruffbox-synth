use crate::building_blocks::{Modulator, SampleBuffer, SynthParameterLabel, SynthParameterValue};

use std::f32::consts::PI;

pub struct BalChan<const BUFSIZE: usize, const NCHAN: usize> {
    levels: [[[f32; BUFSIZE]; NCHAN]; 2],
    pos_mod: Option<Modulator<BUFSIZE>>,
    pos: f32,
}

impl<const BUFSIZE: usize, const NCHAN: usize> Default for BalChan<BUFSIZE, NCHAN> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const BUFSIZE: usize, const NCHAN: usize> BalChan<BUFSIZE, NCHAN> {
    pub fn new() -> Self {
        let mut levels = [[[0.0; BUFSIZE]; NCHAN]; 2];

        levels[0][0] = [0.707; BUFSIZE];
        levels[1][0] = [0.707; BUFSIZE];

        // always start on first channel
        BalChan {
            levels,
            pos_mod: None,
            pos: 0.0,
        }
    }

    pub fn set_modulator(
        &mut self,
        par: SynthParameterLabel,
        init: f32,
        modulator: Modulator<BUFSIZE>,
    ) {
        if par == SynthParameterLabel::ChannelPosition {
            self.pos = init; // keep for later
            self.pos_mod = Some(modulator);
        }
    }

    /// Set the parameter for this panner.
    pub fn set_parameter(&mut self, par: SynthParameterLabel, value: &SynthParameterValue) {
        // if it was more parameters, match would be better,
        // but this way clippy doesn't complain
        if par == SynthParameterLabel::ChannelPosition {
            if let SynthParameterValue::ScalarF32(p) = value {
                self.pos = *p; // keep for later

                self.levels = [[[0.0; BUFSIZE]; NCHAN]; 2];

                let lower = p.floor();
                let angle_rad = (p - lower) * PI * 0.5;
                let upper = lower + 1.0;

                // only two channels are set at any time ...
                self.levels[0][lower as usize % NCHAN] = [angle_rad.cos(); BUFSIZE];
                self.levels[1][upper as usize % NCHAN] = [angle_rad.sin(); BUFSIZE];
            }
        }
    }

    fn recalc_levels(&mut self, start_sample: usize, sample_buffers: &[SampleBuffer]) {
        if self.pos_mod.is_some() {
            self.levels = [[[0.0; BUFSIZE]; NCHAN]; 2];
            let pos_buf =
                self.pos_mod
                    .as_mut()
                    .unwrap()
                    .process(self.pos, start_sample, sample_buffers);
            for (idx, p) in pos_buf.iter().enumerate() {
                let lower = p.floor();
                let angle_rad = (p - lower) * PI * 0.5;
                let upper = lower + 1.0;

                // only two channels are set at any time ...
                self.levels[0][lower as usize % NCHAN][idx] = angle_rad.cos();
                self.levels[1][upper as usize % NCHAN][idx] = angle_rad.sin();
            }
        }
    }

    /// pan mono to stereo
    #[allow(clippy::needless_range_loop)]
    pub fn process_block(
        &mut self,
        block: [[f32; BUFSIZE]; 2],
        start_sample: usize,
        sample_buffers: &[SampleBuffer],
    ) -> [[f32; BUFSIZE]; NCHAN] {
        self.recalc_levels(start_sample, sample_buffers);

        // I think the range loop is way more intuitive and easy to read here ...
        let mut out_buf = [[0.0; BUFSIZE]; NCHAN];
        for c in 0..NCHAN {
            for s in 0..BUFSIZE {
                // this assumes paiwise panning
                out_buf[c][s] =
                    block[0][s] * self.levels[0][c][s] + block[1][s] * self.levels[1][c][s];
            }
        }
        out_buf
    }
}
