use crate::ruffbox::synth::SynthParameter;

/**
 * a simple first order ambisonics encoder
 */
pub struct EncoderO1<const BUFSIZE:usize> {
    a_1_0:f32,
    a_1_1:f32,
    azimuth: f32,
    elevation: f32,
    coefs: [f32; 4],
}

impl <const BUFSIZE:usize> EncoderO1<BUFSIZE> {

    pub fn new() -> Self {
        EncoderO1 {
            a_1_0: 1.0,
            a_1_1: 1.0,
	    azimuth: 0.0,
	    elevation: 0.0,
	    coefs: [0.0; 4]
        }
    }

    /// some parameter limits might be nice ... 
    pub fn set_parameter(&mut self, par: SynthParameter, value: f32) {
        match par {
            SynthParameter::AmbisonicAzimuth => self.azimuth = value,
	    SynthParameter::AmbisonicElevation => self.elevation = value,
            _ => (),
        };

	let sin_a = self.azimuth.sin();
        let cos_a = self.azimuth.cos();
        let sin_e = self.elevation.sin();
        let cos_e = self.elevation.cos();

        self.coefs[0] = 1.0;
        self.coefs[1] = self.a_1_1 * sin_a * sin_e;
        self.coefs[2] = self.a_1_0 * cos_e;
        self.coefs[3] = self.a_1_1 * cos_a * sin_e;
    }
    
    pub fn process_block(&mut self, input: &[f32; BUFSIZE]) -> [[f32; BUFSIZE]; 4] {
        let mut enc_block = [[0.0; BUFSIZE]; 4];

        for i in 0..BUFSIZE {
            enc_block[0][i] = input[i] * self.coefs[0];
            enc_block[1][i] = input[i] * self.coefs[1];
            enc_block[2][i] = input[i] * self.coefs[2];
            enc_block[3][i] = input[i] * self.coefs[3];
        }
        enc_block
    }
}
