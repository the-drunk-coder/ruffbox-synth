use crate::ruffbox::synth::MonoEffect;
use crate::ruffbox::synth::SynthParameter;

/**
 * Three-pole, 18dB/octave filter with tanh distortion
 * Adapted from CSound via Soundpipe
 *
 * My all-time favourite lowpass :D
 * A bit dirty and LoFi
 */
pub struct Lpf18<const BUFSIZE:usize> {
    // user parameters
    cutoff: f32,
    res: f32,
    dist: f32,

    // internal parameters    
    ay1: f32,
    ay2: f32,
    ay11: f32,
    ay31: f32,
    ax1: f32,
    kfcn: f32,
    kp: f32,
    kp1: f32,
    kp1h: f32,
    kres: f32,
    value: f32,
    aout: f32,
    lastin: f32,
    samplerate: f32,
}

impl <const BUFSIZE:usize> Lpf18<BUFSIZE> {
    pub fn new(freq: f32, res: f32, dist: f32, sr: f32) -> Self {
        let kfcn = 2.0 * freq * (1.0 / sr);
        let kp = ((-2.7528 * kfcn + 3.0429) * kfcn + 1.718) * kfcn - 0.9984;
        let kp1 = kp + 1.0;
        let kp1h = 0.5 * kp1;
        let kres = res * (((-2.7079 * kp1 + 10.963) * kp1 - 14.934) * kp1 + 8.4974);
        let value = 1.0 + (dist * (1.5 + 2.0 * res * (1.0 - kfcn)));
        Lpf18 {
            cutoff: freq,
            res: res,
            dist: dist,
            ay1: 0.0,
            ay2: 0.0,
            ax1: 0.0,
            ay11: 0.0,
            ay31: 0.0,            
            kfcn: kfcn,
            kp: kp,
            kp1: kp1,
            kp1h: kp1h,
            kres: kres,
            value: value,
            aout: 0.0,
            lastin: 0.0,
            samplerate: sr,
        }
    }

    pub fn process_sample(&mut self, sample: f32) -> f32 {
        self.ax1  = self.lastin;
        self.ay11 = self.ay1;
        self.ay31 = self.ay2;
        
        self.lastin = sample - (self.kres * self.aout).tanh();
        self.ay1 = self.kp1h * (self.lastin + self.ax1) - self.kp * self.ay1;
        self.ay2 = self.kp1h * (self.ay1 + self.ay11) - self.kp * self.ay2;
        self.aout = self.kp1h * (self.ay2 + self.ay31) - self.kp * self.aout;
        
        (self.aout * self.value).tanh()           
    }
}

impl <const BUFSIZE:usize> MonoEffect<BUFSIZE> for Lpf18<BUFSIZE> {
    // some parameter limits might be nice ... 
    fn set_parameter(&mut self, par: SynthParameter, value: f32) {
        match par {
            SynthParameter::LowpassCutoffFrequency => self.cutoff = value, 
            SynthParameter::LowpassQFactor => self.res = value,
            SynthParameter::LowpassFilterDistortion => self.dist = value,
            _ => (),
        };

        self.kfcn = 2.0 * self.cutoff * (1.0 / self.samplerate);
        self.kp = ((-2.7528 * self.kfcn + 3.0429) * self.kfcn + 1.718) * self.kfcn - 0.9984;
        self.kp1 = self.kp + 1.0;
        self.kp1h = 0.5 * self.kp1;
        self.kres = self.res * (((-2.7079 * self.kp1 + 10.963) * self.kp1 - 14.934) * self.kp1 + 8.4974);
        self.value = 1.0 + (self.dist * (1.5 + 2.0 * self.res * (1.0 - self.kfcn)));
    }
    
    fn finish(&mut self) {} // this effect is stateless
    fn is_finished(&self) -> bool { false } // it's never finished ..

    // start sample isn't really needed either ... 
    fn process_block(&mut self, block: [f32; BUFSIZE], _start_sample: usize) -> [f32; BUFSIZE] {
        let mut out_buf: [f32; BUFSIZE] = [0.0; BUFSIZE];

        for i in 0..BUFSIZE {

            self.ax1  = self.lastin;
            self.ay11 = self.ay1;
            self.ay31 = self.ay2;

            self.lastin = block[i] - (self.kres * self.aout).tanh();
            self.ay1 = self.kp1h * (self.lastin + self.ax1) - self.kp * self.ay1;
            self.ay2 = self.kp1h * (self.ay1 + self.ay11) - self.kp * self.ay2;
            self.aout = self.kp1h * (self.ay2 + self.ay31) - self.kp * self.aout;

            out_buf[i] = (self.aout * self.value).tanh();            
        }

        out_buf
    }
}

/**
 * Biquad HiPass Filter, 12dB/oct
 */
pub struct BiquadHpf<const BUFSIZE:usize> {
    // user parameters
    cutoff: f32,
    q: f32,
    
    // internal parameters    
    a1: f32,
    a2: f32,
    b0: f32,
    b1: f32,
    b2: f32,
    del1: f32,
    del2: f32,
    k: f32,
    samplerate: f32	
}

impl <const BUFSIZE:usize> BiquadHpf<BUFSIZE> {
    pub fn new(freq: f32, q: f32, sr: f32) -> Self {
	let k = ((std::f32::consts::PI * freq) / sr).tanh();
	let k_pow_two = k.powf(2.0);
	let b0 = q / ((k_pow_two * q) + k + q);
        BiquadHpf {
            cutoff: freq,
            q: q,
            a1: (2.0 * q * (k_pow_two - 1.0)) / ((k_pow_two * q) + k + q),
            a2: ((k_pow_two * q) - k + q) / ((k_pow_two * q) + k + q),
            b0: b0,
	    b1: -1.0 * ((2.0 * q) / ((k_pow_two * q) + k + q)),
	    b2: b0,
	    del1: 0.0,
	    del2: 0.0,
            k: k,
            samplerate: sr,
        }
    }
}

impl <const BUFSIZE:usize> MonoEffect<BUFSIZE> for BiquadHpf<BUFSIZE> {
    // some parameter limits might be nice ... 
    fn set_parameter(&mut self, par: SynthParameter, value: f32) {
        match par {
            SynthParameter::HighpassCutoffFrequency => self.cutoff = value, 
            SynthParameter::HighpassQFactor => self.q = value,            
            _ => (),
        };

	// reset delay
	self.del1 = 0.0;
	self.del2 = 0.0;

	self.k = ((std::f32::consts::PI * self.cutoff) / self.samplerate).tanh();
	let k_pow_two = self.k.powf(2.0);
	self.a1 = (2.0 * self.q * (k_pow_two - 1.0)) / ((k_pow_two * self.q) + self.k + self.q);
	self.a2 = ((k_pow_two * self.q) - self.k + self.q) / ((k_pow_two * self.q) + self.k + self.q);
	self.b0 = self.q / ((k_pow_two * self.q) + self.k + self.q);
	self.b1 = -1.0 * ((2.0 * self.q) / ((k_pow_two * self.q) + self.k + self.q));
	self.b2 = self.b0;        
    }
    
    fn finish(&mut self) {} // this effect is stateless
    fn is_finished(&self) -> bool { false } // it's never finished ..

    // start sample isn't really needed either ... 
    fn process_block(&mut self, block: [f32; BUFSIZE], _start_sample: usize) -> [f32; BUFSIZE] {
        let mut out_buf: [f32; BUFSIZE] = [0.0; BUFSIZE];

        for i in 0..BUFSIZE {	    
            let intermediate = block[i] + ((-1.0 * self.a1) * self.del1) + ((-1.0 * self.a2) * self.del2);
	    out_buf[i] = (self.b0 * intermediate) + (self.b1 * self.del1) + (self.b2 * self.del2);
	    self.del2 = self.del1;
	    self.del1 = intermediate;           
        }

        out_buf
    }
}


/**
 * Peak/Eq Filter
 */
pub struct PeakEq<const BUFSIZE:usize> {
    // user parameters
    center: f32,
    bw: f32,
    gain: f32,
    
    // internal parameters    
    h_zero: f32,
    v_zero: f32,
    d: f32,
    del1: f32,
    del2: f32,
    c: f32,
    samplerate: f32	
}

impl <const BUFSIZE:usize> PeakEq<BUFSIZE> {
    pub fn new(center_freq: f32, bw: f32, gain: f32, sr: f32) -> Self {
	
	let w_c = (2.0 * center_freq) / sr;	
	let w_b = (2.0 * bw) / sr;
	let d = -((std::f32::consts::PI * w_c).cos());
	let v_zero = (gain / 20.0).powf(10.0);
	let h_zero = v_zero - 1.0;
	let cf_tan = (std::f32::consts::PI * w_b / 2.0).tan();

	let c = if gain >= 0.0 {
	    (cf_tan - 1.0) / (cf_tan + 1.0)
	} else {
	    (cf_tan - v_zero) / (cf_tan + v_zero)
	};
		
        PeakEq {	    
	    center: center_freq,
	    bw: bw,
	    gain: gain,	    
	    h_zero: h_zero,
	    v_zero: v_zero,
	    d: d,
	    del1: 0.0,
	    del2: 0.0,
	    c: c,
	    samplerate: sr	
	}
    }
}

impl <const BUFSIZE:usize> MonoEffect<BUFSIZE> for PeakEq<BUFSIZE> {
    // some parameter limits might be nice ... 
    fn set_parameter(&mut self, par: SynthParameter, value: f32) {
        match par {
            SynthParameter::PeakFrequency => self.center = value, 
            SynthParameter::PeakGain => self.gain = value,
	    SynthParameter::PeakQFactor => self.bw = value,            
            _ => (),
        };

	// reset delay
	self.del1 = 0.0;
	self.del2 = 0.0;

	let w_c = (2.0 * self.center) / self.samplerate;	
	let w_b = (2.0 * self.bw) / self.samplerate;
	self.d = -((std::f32::consts::PI * w_c).cos());
	self.v_zero = (self.gain / 20.0).powf(10.0);
	self.h_zero = self.v_zero - 1.0;
	let cf_tan = (std::f32::consts::PI * w_b / 2.0).tan();

	self.c = if self.gain >= 0.0 {
	    (cf_tan - 1.0) / (cf_tan + 1.0)
	} else {
	    (cf_tan - self.v_zero) / (cf_tan + self.v_zero)
	};
    }
    
    fn finish(&mut self) {} // this effect is stateless
    fn is_finished(&self) -> bool { false } // it's never finished ..

    // start sample isn't really needed either ... 
    fn process_block(&mut self, block: [f32; BUFSIZE], _start_sample: usize) -> [f32; BUFSIZE] {
        let mut out_buf: [f32; BUFSIZE] = [0.0; BUFSIZE];

        for i in 0..BUFSIZE {	    
            let x_h = block[i] - self.d * (1.0 - self.c) * self.del1 + (self.c * self.del2);
	    let y_one = (-1.0 * self.c * x_h) + (self.d * (1.0 - self.c) * self.del1) + self.del2;
	    out_buf[i] = 0.5 * self.h_zero * (block[i] - y_one) + block[i];
	    self.del2 = self.del1;
	    self.del1 = x_h;
        }

        out_buf
    }
}


