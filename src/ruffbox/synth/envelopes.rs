use crate::ruffbox::synth::MonoEffect;
use crate::ruffbox::synth::SynthParameter;
use crate::ruffbox::synth::SynthState;

/// simple linear attack-sustain-release envelope
#[derive(Clone,Copy)]
pub struct ASREnvelope<const BUFSIZE: usize> {
    samplerate: f32,
    atk: f32,
    sus: f32,
    rel: f32,
    atk_samples: usize,
    sus_samples: usize,
    rel_samples: usize,
    sample_count: usize,
    lvl: f32,
    max_lvl: f32,
    atk_lvl_increment: f32,
    rel_lvl_decrement: f32,
    state: SynthState,
}

impl<const BUFSIZE: usize> ASREnvelope<BUFSIZE> {
    pub fn new(samplerate: f32, lvl: f32, atk: f32, sus: f32, rel: f32) -> Self {
        let atk_samples = (samplerate * atk).round();
        let sus_samples = atk_samples + (samplerate * sus).round();
        let rel_samples = sus_samples + (samplerate * rel).round();

        //println!("atk sam: {} sus sam: {} rel sam: {}", atk_samples.round(), sus_samples.round(), rel_samples.round());

        ASREnvelope {
            samplerate: samplerate,
            atk: atk,
            sus: sus,
            rel: rel,
            atk_samples: atk_samples as usize,
            sus_samples: sus_samples as usize,
            rel_samples: rel_samples as usize,
            sample_count: 0,
            lvl: 0.0,
            max_lvl: lvl,
            atk_lvl_increment: lvl / atk_samples,
            rel_lvl_decrement: lvl / (rel_samples - sus_samples),
            state: SynthState::Fresh,
        }
    }
}

impl<const BUFSIZE: usize> MonoEffect<BUFSIZE> for ASREnvelope<BUFSIZE> {
    fn finish(&mut self) {
        self.state = SynthState::Finished;
    }

    fn is_finished(&self) -> bool {
        match self.state {
            SynthState::Finished => true,
            _ => false,
        }
    }

    fn set_parameter(&mut self, par: SynthParameter, value: f32) {
        let mut update_internals = false;
        match par {
            SynthParameter::Attack => {
                self.atk = value;
                update_internals = true;
            }
            SynthParameter::Sustain => {
                self.sus = value;
                update_internals = true;
            }
            SynthParameter::Release => {
                self.rel = value;
                update_internals = true;
            }
            SynthParameter::Level => {
                self.max_lvl = value;
                update_internals = true;
            }
            SynthParameter::Samplerate => {
                self.samplerate = value;
                update_internals = true;
            }
            _ => (),
        };

        if update_internals {
            self.atk_samples = (self.samplerate * self.atk).round() as usize;
            self.sus_samples = self.atk_samples + (self.samplerate * self.sus).round() as usize;
            self.rel_samples = self.sus_samples + (self.samplerate * self.rel).round() as usize;

            // keep values sane
            self.atk_lvl_increment = self.max_lvl / self.atk_samples as f32;
            if self.atk_lvl_increment != 0.0 && !self.atk_lvl_increment.is_normal() {
                self.atk_lvl_increment = 0.0;
            }

            self.rel_lvl_decrement = self.max_lvl / (self.rel_samples - self.sus_samples) as f32;
            if self.rel_lvl_decrement != 0.0 && !self.rel_lvl_decrement.is_normal() {
                self.rel_lvl_decrement = 0.0;
            }

            // println!("atk sam: {} sus sam: {} rel sam: {} atk inc: {} rel dec: {}",
            //        self.atk_samples,
            //        self.sus_samples,
            //        self.rel_samples,
            //        self.atk_lvl_increment,
            //        self.rel_lvl_decrement);
        }
    }

    fn process_block(&mut self, block: [f32; BUFSIZE], start_sample: usize) -> [f32; BUFSIZE] {
        let mut out: [f32; BUFSIZE] = [0.0; BUFSIZE];

        for i in start_sample..BUFSIZE {
            out[i] = block[i] * self.lvl;

            self.sample_count += 1;
            if self.sample_count < self.atk_samples {
                self.lvl += self.atk_lvl_increment;
            } else if self.sample_count >= self.atk_samples && self.sample_count < self.sus_samples
            {
                self.lvl = self.max_lvl;
            } else if self.sample_count >= self.sus_samples
                && self.sample_count < self.rel_samples - 1
            {
                self.lvl -= self.rel_lvl_decrement;
            } else if self.sample_count >= self.rel_samples - 1 {
                self.lvl = 0.0;
                self.finish();
            }
        }
        out
    }
}


/// Exponential/Linear Percussion Envelope (currently with fixed curve value)
#[derive(Clone,Copy)]
pub struct ExpPercEnvelope<const BUFSIZE: usize> {
    samplerate: f32,    
    atk: f32,
    sus: f32,
    rel: f32,
    atk_samples: usize,
    sus_samples: usize,
    rel_samples: usize,
    atk_inc: f32,
    rel_inc: f32,
    time_count: f32,
    curve: f32,
    sample_count: usize,
    max_lvl: f32,    
    state: SynthState,
}

impl<const BUFSIZE: usize> ExpPercEnvelope<BUFSIZE> {
    pub fn new(samplerate: f32, lvl: f32, atk: f32, sus: f32, rel: f32) -> Self {
        let atk_samples = (samplerate * atk).round() as usize;
        let sus_samples = atk_samples + (samplerate * sus).round() as usize;
        let rel_samples = (samplerate * rel).round() as usize;
	
	let atk_inc = 1.0 / atk_samples as f32;
	let rel_inc = 1.0 / rel_samples as f32;

	//println!("atk sam: {} sus sam: {} rel sam: {}", atk_samples, sus_samples, rel_samples );
					
        ExpPercEnvelope {
            samplerate: samplerate,
            atk: atk,
            sus: sus,
            rel: rel,
            atk_samples: atk_samples,
            sus_samples: sus_samples,
            rel_samples: sus_samples + rel_samples,
            sample_count: 0,
	    atk_inc: atk_inc,
	    rel_inc: rel_inc,
	    curve: -4.5,
	    time_count: 0.0,
            max_lvl: lvl,            
            state: SynthState::Fresh,
        }
    }
}

impl<const BUFSIZE: usize> MonoEffect<BUFSIZE> for ExpPercEnvelope<BUFSIZE> {
    fn finish(&mut self) {
        self.state = SynthState::Finished;
    }

    fn is_finished(&self) -> bool {
        match self.state {
            SynthState::Finished => true,
            _ => false,
        }
    }

    fn set_parameter(&mut self, par: SynthParameter, value: f32) {
        let mut update_internals = false;
        match par {
            SynthParameter::Attack => {
                self.atk = value;
                update_internals = true;
            }
            SynthParameter::Sustain => {
                self.sus = value;
                update_internals = true;
            }
            SynthParameter::Release => {
                self.rel = value;
                update_internals = true;
            }
            SynthParameter::Level => {
                self.max_lvl = value;             
            }
            SynthParameter::Samplerate => {
                self.samplerate = value;
                update_internals = true;
            }
            _ => (),
        };

        if update_internals {
            self.atk_samples = (self.samplerate * self.atk).round() as usize;
            self.sus_samples = self.atk_samples + (self.samplerate * self.sus).round() as usize;
            self.rel_samples = (self.samplerate * self.rel).round() as usize;

	    self.atk_inc = 1.0 / self.atk_samples as f32;
	    self.rel_inc = 1.0 / self.rel_samples as f32;
	    
            self.rel_samples += self.sus_samples;
        }
    }

    fn process_block(&mut self, block: [f32; BUFSIZE], start_sample: usize) -> [f32; BUFSIZE] {
        let mut out: [f32; BUFSIZE] = [0.0; BUFSIZE];

	for i in start_sample..BUFSIZE {
	    
            let env = if self.sample_count < self.atk_samples {
		let env = ((self.curve * self.time_count).exp() - 1.0) / (self.curve.exp() - 1.0);
		self.time_count += self.atk_inc;
		env		    
            } else if self.sample_count >= self.atk_samples && self.sample_count <= self.sus_samples
            {
		self.time_count = 0.0; // this is a bit redundant ...
		1.0
            } else if self.sample_count > self.sus_samples
                && self.sample_count < self.rel_samples - 1
            {
                let env = ((self.curve * self.time_count).exp() - 1.0) / (self.curve.exp() - 1.0);
		self.time_count += self.rel_inc;
		1.0 - env
            } else {
                self.finish();
		0.0
            };

	    out[i] = block[i] * env * self.max_lvl;

            self.sample_count += 1;	    
        }
        
        out
    }
}

// TEST TEST TEST
#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    /// test the general workings of the ASREnvelope
    #[test]
    fn test_asr_envelope() {
        let test_block: [f32; 128] = [1.0; 128];

        // half a block attack, one block sustain, half a block release ... 2 blocks total .
        let mut env = ASREnvelope::<128>::new(44100.0, 0.5, 0.0014512, 0.0029024, 0.0014512);

        let out_1: [f32; 128] = env.process_block(test_block, 0);
        let out_2: [f32; 128] = env.process_block(test_block, 0);

        // comparison
        let mut comp_block_1: [f32; 128] = [0.0; 128];
        let mut comp_block_2: [f32; 128] = [0.0; 128];

        let mut gain = 0.0;
        let gain_inc_dec = 0.5 / 64.0;

        // fill comp blocks
        for i in 0..64 {
            comp_block_1[i] = test_block[i] * gain;
            gain += gain_inc_dec;
        }

        for i in 0..64 {
            comp_block_1[64 + i] = test_block[64 + i] * gain;
        }

        for i in 0..64 {
            comp_block_2[i] = test_block[i] * gain;
        }

        for i in 0..64 {
            gain -= gain_inc_dec;
            comp_block_2[64 + i] = test_block[64 + i] * gain;
        }

        for i in 0..128 {
            //println!("{} {}", out_1[i], comp_block_1[i]);
            assert_approx_eq::assert_approx_eq!(out_1[i], comp_block_1[i], 0.00001);
        }

        for i in 0..128 {
            //println!("{} {}", out_2[i], comp_block_2[i]);
            assert_approx_eq::assert_approx_eq!(out_2[i], comp_block_2[i], 0.00001);
        }
    }

    /// test the parameter setter of the envelope
    #[test]
    fn test_asr_envelope_set_params() {
        let test_block: [f32; 128] = [1.0; 128];

        // half a block attack, one block sustain, half a block release ... 2 blocks total .
        let mut env = ASREnvelope::<128>::new(44100.0, 0.0, 0.0, 0.0, 0.0);

        // use paramter setter to set parameters ...
        env.set_parameter(SynthParameter::Attack, 0.0014512);
        env.set_parameter(SynthParameter::Sustain, 0.0029024);
        env.set_parameter(SynthParameter::Release, 0.0014512);
        env.set_parameter(SynthParameter::Level, 0.5);

        let out_1: [f32; 128] = env.process_block(test_block, 0);
        let out_2: [f32; 128] = env.process_block(test_block, 0);

        // comparison
        let mut comp_block_1: [f32; 128] = [0.0; 128];
        let mut comp_block_2: [f32; 128] = [0.0; 128];

        let mut gain = 0.0;
        let gain_inc_dec = 0.5 / 64.0;

        // fill comp blocks
        for i in 0..64 {
            comp_block_1[i] = test_block[i] * gain;
            gain += gain_inc_dec;
        }

        for i in 0..64 {
            comp_block_1[64 + i] = test_block[64 + i] * gain;
        }

        for i in 0..64 {
            comp_block_2[i] = test_block[i] * gain;
        }

        for i in 0..64 {
            gain -= gain_inc_dec;
            comp_block_2[64 + i] = test_block[64 + i] * gain;
        }

        for i in 0..128 {
            //println!("{} {}", out_1[i], comp_block_1[i]);
            assert_approx_eq::assert_approx_eq!(out_1[i], comp_block_1[i], 0.00001);
        }

        for i in 0..128 {
            //println!("{} {}", out_2[i], comp_block_2[i]);
            assert_approx_eq::assert_approx_eq!(out_2[i], comp_block_2[i], 0.00001);
        }
    }

    #[test]
    fn test_asr_envelope_short_intervals_with_offset() {
        let test_block: [f32; 128] = [1.0; 128];

        // let this one start at the beginning of a block
        let mut env_at_start = ASREnvelope::<128>::new(44100.0, 0.0, 0.0, 0.0, 0.0);
        // let this one start somewhere in the block
        let mut env_with_offset = ASREnvelope::<128>::new(44100.0, 0.0, 0.0, 0.0, 0.0);

        // use paramter setter to set parameters ...
        println!("Set parameters for env at start:");
        env_at_start.set_parameter(SynthParameter::Level, 1.0);
        env_at_start.set_parameter(SynthParameter::Attack, 0.001);
        env_at_start.set_parameter(SynthParameter::Sustain, 0.019);
        env_at_start.set_parameter(SynthParameter::Release, 0.07);

        println!("\nSet parameters for env with offset:");
        env_with_offset.set_parameter(SynthParameter::Level, 1.0);
        env_with_offset.set_parameter(SynthParameter::Attack, 0.001);
        env_with_offset.set_parameter(SynthParameter::Sustain, 0.019);
        env_with_offset.set_parameter(SynthParameter::Release, 0.07);

        println!("");

        let mut out_start = env_at_start.process_block(test_block, 0);
        let mut out_offset = env_with_offset.process_block(test_block, 60);

        // calculate 34 blocks
        for _ in 0..34 {
            for i in 0..68 {
                //print!("{} {} - ", out_start[i], out_offset[i + 60]);
                assert_approx_eq::assert_approx_eq!(out_start[i], out_offset[i + 60], 0.00001);
            }
            //println!{" block {}.1 done \n", i};

            out_offset = env_with_offset.process_block(test_block, 0);

            for i in 68..128 {
                //print!("{} {} - ", out_start[i], out_offset[i - 68]);
                assert_approx_eq::assert_approx_eq!(out_start[i], out_offset[i - 68], 0.00001);
            }

            //println!{" block {}.2 done \n", i};
            out_start = env_at_start.process_block(test_block, 0);
        }
    }

    #[test]
    fn perc_exp_smoke_test() {
	let mut exp_env = ExpPercEnvelope::<128>::new(16000.0, 1.0, 0.05, 0.0, 1.0);
	let test_block: [f32; 128] = [1.0; 128];
	let mut out = Vec::new();
	for _ in 0..132 {
	    let env_out = exp_env.process_block(test_block, 0);
	    out.extend_from_slice(&env_out);
	}
		
	assert_approx_eq::assert_approx_eq!(out[0], 0.0, 0.00001);
	assert_approx_eq::assert_approx_eq!(out[800], 1.0, 0.00001);
	assert_approx_eq::assert_approx_eq!(out[16800], 0.0, 0.00001);

	for sample in out.iter() {
	    assert!(*sample >= 0.0);
	    assert!(*sample <= 1.0);
	}
    }
}
