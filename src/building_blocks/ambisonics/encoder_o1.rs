use crate::building_blocks::{Modulator, SampleBuffer, SynthParameterLabel, SynthParameterValue};

/**
 * a simple first order ambisonics encoder
 */
pub struct EncoderO1<const BUFSIZE: usize> {
    a_1_0: f32,
    a_1_1: f32,
    azimuth: f32,
    elevation: f32,
    azimuth_mod: Option<Modulator<BUFSIZE>>,
    elevation_mod: Option<Modulator<BUFSIZE>>,
    coefs: [[f32; BUFSIZE]; 4],
}

impl<const BUFSIZE: usize> Default for EncoderO1<BUFSIZE> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const BUFSIZE: usize> EncoderO1<BUFSIZE> {
    pub fn new() -> Self {
        EncoderO1 {
            a_1_0: 1.0,
            a_1_1: 1.0,
            azimuth: 0.0,
            elevation: -std::f32::consts::PI / 2.0,
            azimuth_mod: None,
            elevation_mod: None,
            coefs: [[0.0; BUFSIZE]; 4],
        }
    }

    pub fn set_modulator(
        &mut self,
        par: SynthParameterLabel,
        init: f32,
        modulator: Modulator<BUFSIZE>,
    ) {
        match par {
            SynthParameterLabel::AmbisonicAzimuth => {
                self.azimuth = init; // keep for later
                self.azimuth_mod = Some(modulator);
            }
            SynthParameterLabel::AmbisonicElevation => {
                self.elevation = init - std::f32::consts::PI / 2.0; // keep for later
                self.elevation_mod = Some(modulator);
            }
            _ => {}
        }
    }

    // some parameter limits might be nice ...
    pub fn set_parameter(&mut self, par: SynthParameterLabel, value: &SynthParameterValue) {
        if let SynthParameterValue::ScalarF32(val) = value {
            match par {
                SynthParameterLabel::AmbisonicAzimuth => self.azimuth = *val,
                SynthParameterLabel::AmbisonicElevation => {
                    self.elevation = *val - std::f32::consts::PI / 2.0
                }
                _ => {}
            };

            let sin_a = self.azimuth.sin();
            let cos_a = self.azimuth.cos();
            let sin_e = self.elevation.sin();
            let cos_e = self.elevation.cos();

            self.coefs[0] = [1.0; BUFSIZE];
            self.coefs[1] = [self.a_1_1 * sin_a * sin_e; BUFSIZE];
            self.coefs[2] = [self.a_1_0 * cos_e; BUFSIZE];
            self.coefs[3] = [self.a_1_1 * cos_a * sin_e; BUFSIZE];
        }
    }

    pub fn process_block(
        &mut self,
        input: [f32; BUFSIZE],
        start_sample: usize,
        in_buffers: &[SampleBuffer],
    ) -> [[f32; BUFSIZE]; 4] {
        let mut enc_block = [[0.0; BUFSIZE]; 4];

        if self.azimuth_mod.is_some() || self.elevation_mod.is_some() {
            let azi_buf = if let Some(azi_mod) = self.azimuth_mod.as_mut() {
                azi_mod.process(self.azimuth, start_sample, in_buffers)
            } else {
                [self.azimuth; BUFSIZE]
            };
            let ele_buf = if let Some(ele_mod) = self.elevation_mod.as_mut() {
                ele_mod
                    .process(
                        self.elevation + std::f32::consts::PI / 2.0,
                        start_sample,
                        in_buffers,
                    )
                    .map(|x| x - std::f32::consts::PI / 2.0)
            } else {
                [self.elevation; BUFSIZE]
            };
            self.coefs[0] = [1.0; BUFSIZE];

            for s in 0..BUFSIZE {
                let sin_a = azi_buf[s].sin();
                let cos_a = azi_buf[s].cos();
                let sin_e = ele_buf[s].sin();
                let cos_e = ele_buf[s].cos();

                self.coefs[1][s] = self.a_1_1 * sin_a * sin_e;
                self.coefs[2][s] = self.a_1_0 * cos_e;
                self.coefs[3][s] = self.a_1_1 * cos_a * sin_e;
            }
        }

        for (i, input_sample) in input.iter().enumerate().take(BUFSIZE) {
            enc_block[0][i] = input_sample * self.coefs[0][i];
            enc_block[1][i] = input_sample * self.coefs[1][i];
            enc_block[2][i] = input_sample * self.coefs[2][i];
            enc_block[3][i] = input_sample * self.coefs[3][i];
        }
        enc_block
    }
}
