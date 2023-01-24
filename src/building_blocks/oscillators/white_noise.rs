use crate::building_blocks::{Modulator, MonoSource, SynthParameterLabel, SynthParameterValue};

/**
 * a white noise generator based on wyrand (through fastrand)
 */
#[derive(Clone)]
pub struct WhiteNoise<const BUFSIZE: usize> {
    amp: f32,
    amp_mod: Option<Modulator<BUFSIZE>>, // and level
}

impl<const BUFSIZE: usize> WhiteNoise<BUFSIZE> {
    pub fn new(amp: f32) -> Self {
        WhiteNoise { amp, amp_mod: None }
    }
}

impl<const BUFSIZE: usize> MonoSource<BUFSIZE> for WhiteNoise<BUFSIZE> {
    fn reset(&mut self) {}

    fn set_modulator(
        &mut self,
        par: SynthParameterLabel,
        init: f32,
        modulator: Modulator<BUFSIZE>,
    ) {
        if par == SynthParameterLabel::OscillatorAmplitude {
            self.amp = init;
            self.amp_mod = Some(modulator);
        }
    }

    fn set_parameter(&mut self, par: SynthParameterLabel, value: &SynthParameterValue) {
        if par == SynthParameterLabel::OscillatorAmplitude {
            if let SynthParameterValue::ScalarF32(l) = value {
                self.amp = *l;
            }
        }
    }

    fn finish(&mut self) {}

    fn is_finished(&self) -> bool {
        false
    }

    fn get_next_block(
        &mut self,
        start_sample: usize,
        in_buffers: &[SampleBuffer],
    ) -> [f32; BUFSIZE] {
        let mut out_buf: [f32; BUFSIZE] = [0.0; BUFSIZE];

        if let Some(m) = self.amp_mod.as_mut() {
            let amp_buf = m.process(self.amp, start_sample, in_buffers);

            for (idx, current_sample) in out_buf
                .iter_mut()
                .enumerate()
                .take(BUFSIZE)
                .skip(start_sample)
            {
                let raw = fastrand::i32(-100..100);
                *current_sample = (raw as f32 / 100.0) * amp_buf[idx];
            }
        } else {
            for current_sample in out_buf.iter_mut().take(BUFSIZE).skip(start_sample) {
                let raw = fastrand::i32(-100..100);
                *current_sample = (raw as f32 / 100.0) * self.amp;
            }
        }

        out_buf
    }
}
