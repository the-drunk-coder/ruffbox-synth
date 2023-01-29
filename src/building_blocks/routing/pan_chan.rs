use crate::building_blocks::{Modulator, SampleBuffer, SynthParameterLabel, SynthParameterValue};

use std::f32::consts::PI;

pub struct PanChan<const BUFSIZE: usize, const NCHAN: usize> {
    levels: [[f32; BUFSIZE]; NCHAN],
    pos_mod: Option<Modulator<BUFSIZE>>,
    pos: f32,
}

impl<const BUFSIZE: usize, const NCHAN: usize> Default for PanChan<BUFSIZE, NCHAN> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const BUFSIZE: usize, const NCHAN: usize> PanChan<BUFSIZE, NCHAN> {
    pub fn new() -> Self {
        let mut lvls = [[0.0; BUFSIZE]; NCHAN];
        lvls[0] = [1.0; BUFSIZE];
        // always start on first channel
        PanChan {
            levels: lvls,
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

                self.levels = [[0.0; BUFSIZE]; NCHAN];

                let lower = p.floor();
                let angle_rad = (p - lower) * PI * 0.5;
                let upper = lower + 1.0;

                self.levels[lower as usize % (NCHAN as usize)] = [angle_rad.cos(); BUFSIZE];
                self.levels[upper as usize % (NCHAN as usize)] = [angle_rad.sin(); BUFSIZE];
            }
        }
    }

    fn recalc_levels(&mut self, start_sample: usize, sample_buffers: &[SampleBuffer]) {
        if self.pos_mod.is_some() {
            self.levels = [[0.0; BUFSIZE]; NCHAN];
            let pos_buf =
                self.pos_mod
                    .as_mut()
                    .unwrap()
                    .process(self.pos, start_sample, sample_buffers);
            for (idx, p) in pos_buf.iter().enumerate() {
                let lower = p.floor();
                let angle_rad = (p - lower) * PI * 0.5;
                let upper = lower + 1.0;

                self.levels[lower as usize % (NCHAN as usize)][idx] = angle_rad.cos();
                self.levels[upper as usize % (NCHAN as usize)][idx] = angle_rad.sin();
            }
        }
    }

    /// pan mono to stereo
    #[allow(clippy::needless_range_loop)]
    pub fn process_block(
        &mut self,
        block: [f32; BUFSIZE],
        start_sample: usize,
        sample_buffers: &[SampleBuffer],
    ) -> [[f32; BUFSIZE]; NCHAN] {
        self.recalc_levels(start_sample, sample_buffers);

        // I think the range loop is way more intuitive and easy to read here ...
        let mut out_buf = [[0.0; BUFSIZE]; NCHAN];
        for c in 0..NCHAN {
            for s in 0..BUFSIZE {
                out_buf[c][s] = block[s] * self.levels[c][s];
            }
        }
        out_buf
    }
}
