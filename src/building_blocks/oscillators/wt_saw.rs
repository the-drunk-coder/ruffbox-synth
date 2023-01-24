use crate::building_blocks::{
    oscillators::Wavetable, Modulator, MonoSource, SampleBuffer, SynthParameterLabel,
    SynthParameterValue,
};

/**
 * A band-limited wavetable-based sawtooth oscillator.
 */
#[derive(Clone)]
pub struct WTSaw<const BUFSIZE: usize> {
    wt: Wavetable<BUFSIZE>,
}

impl<const BUFSIZE: usize> WTSaw<BUFSIZE> {
    pub fn new(freq: f32, amp: f32, samplerate: f32) -> Self {
        let mut wt = Wavetable::new(samplerate);

        let mut tab = vec![0.0; 2048];

        for (i, sample) in tab.iter_mut().enumerate().take(2048) {
            *sample = -1.0 + ((2.0 / 2048.0) * i as f32)
        }

        wt.set_parameter(
            SynthParameterLabel::Wavetable,
            &SynthParameterValue::VecF32(tab),
        );

        wt.set_parameter(
            SynthParameterLabel::PitchFrequency,
            &SynthParameterValue::ScalarF32(freq),
        );

        wt.set_parameter(
            SynthParameterLabel::OscillatorAmplitude,
            &SynthParameterValue::ScalarF32(amp),
        );

        WTSaw { wt }
    }
}

impl<const BUFSIZE: usize> MonoSource<BUFSIZE> for WTSaw<BUFSIZE> {
    fn reset(&mut self) {}

    fn set_modulator(
        &mut self,
        par: SynthParameterLabel,
        init: f32,
        modulator: Modulator<BUFSIZE>,
    ) {
        self.wt.set_modulator(par, init, modulator);
    }

    fn set_parameter(&mut self, par: SynthParameterLabel, val: &SynthParameterValue) {
        self.wt.set_parameter(par, val);
    }

    fn finish(&mut self) {
        self.wt.finish();
    }

    fn is_finished(&self) -> bool {
        self.wt.is_finished()
    }

    fn get_next_block(
        &mut self,
        start_sample: usize,
        in_buffers: &[SampleBuffer],
    ) -> [f32; BUFSIZE] {
        self.wt.get_next_block(start_sample, in_buffers)
    }
}
