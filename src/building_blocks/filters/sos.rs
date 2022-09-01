// generic second-order section
pub struct SOSCoefs {
    // internal parameters
    pub a1: f32,
    pub a2: f32,
    pub b0: f32,
    pub b1: f32,
    pub b2: f32,
}

pub struct SOSDelay {
    pub del1: f32,
    pub del2: f32,
}

impl Default for SOSCoefs {
    fn default() -> Self {
        SOSCoefs {
            a1: 0.0,
            a2: 0.0,
            b0: 0.0,
            b1: 0.0,
            b2: 0.0,
        }
    }
}

impl Default for SOSDelay {
    fn default() -> Self {
        SOSDelay {
            del1: 0.0,
            del2: 0.0,
        }
    }
}

pub fn process_sos_block<const BUFSIZE: usize>(
    coefs: &SOSCoefs,
    delay: &mut SOSDelay,
    block: [f32; BUFSIZE],
) -> [f32; BUFSIZE] {
    let mut out_buf: [f32; BUFSIZE] = [0.0; BUFSIZE];
    let mut intermediate: f32;
    for i in 0..BUFSIZE {
        intermediate =
            block[i] + ((-1.0 * coefs.a1) * delay.del1) + ((-1.0 * coefs.a2) * delay.del2);
        out_buf[i] = (coefs.b0 * intermediate) + (coefs.b1 * delay.del1) + (coefs.b2 * delay.del2);
        delay.del2 = delay.del1;
        delay.del1 = intermediate;
    }
    out_buf
}

#[inline(always)]
pub fn process_sos_sample(coefs: &SOSCoefs, delay: &mut SOSDelay, sample: f32) -> f32 {
    let intermediate = sample + ((-1.0 * coefs.a1) * delay.del1) + ((-1.0 * coefs.a2) * delay.del2);
    let out = (coefs.b0 * intermediate) + (coefs.b1 * delay.del1) + (coefs.b2 * delay.del2);
    delay.del2 = delay.del1;
    delay.del1 = intermediate;
    out
}
