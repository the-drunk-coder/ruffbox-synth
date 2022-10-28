use crate::building_blocks::envelopes::*;
use crate::building_blocks::filters::*;
use crate::building_blocks::oscillators::*;
use crate::building_blocks::routing::PanChan;
use crate::building_blocks::{
    Modulator, MonoEffect, MonoSource, Synth, SynthParameterLabel, SynthParameterValue,
    ValueOrModulator,
};

/// 11-partial risset bell, modeled after Frederik Oloffson's SuperCollider port
pub struct RissetBell<const BUFSIZE: usize, const NCHAN: usize> {
    oscillators: [SineOsc<BUFSIZE>; 11],
    envelopes: [ExpPercEnvelope<BUFSIZE>; 11],
    main_envelope: LinearASREnvelope<BUFSIZE>,
    amps: [f32; 11],
    durs: [f32; 11],
    freqs: [f32; 11],
    dets: [f32; 11],
    lpf: Lpf18<BUFSIZE>,
    balance: PanChan<BUFSIZE, NCHAN>,
    atk: f32,
    sus: f32,
    rel: f32,
    freq: f32,
    main_level: f32,
    reverb: f32,
    delay: f32,
}

impl<const BUFSIZE: usize, const NCHAN: usize> RissetBell<BUFSIZE, NCHAN> {
    pub fn new(sr: f32) -> RissetBell<BUFSIZE, NCHAN> {
        let mut bell = RissetBell {
            oscillators: [(); 11].map(|_| SineOsc::new(440.0, 0.4, sr)),
            envelopes: [ExpPercEnvelope::new(1.0, 0.005, 0.0, 0.05, sr); 11],
            main_envelope: LinearASREnvelope::new(1.0, 0.05, 0.5, 0.05, sr),
            amps: [1.0, 0.67, 1.0, 1.8, 2.67, 1.67, 1.46, 1.33, 1.33, 1.0, 1.33],
            durs: [
                1.0, 0.9, 0.65, 0.55, 0.325, 0.35, 0.25, 0.2, 0.15, 0.1, 0.075,
            ],
            freqs: [
                0.56, 0.56, 0.92, 0.92, 1.19, 1.7, 2.0, 2.74, 3.0, 3.76, 4.07,
            ],
            dets: [0.0, 1.0, 0.0, 1.7, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            lpf: Lpf18::new(19500.0, 0.01, 0.01, sr),
            balance: PanChan::new(),
            atk: 0.05,
            sus: 0.7,
            rel: 0.05,
            main_level: 0.7,
            freq: 1000.0,
            reverb: 0.0,
            delay: 0.0,
        };

        // init with some default frequency
        let freq = 1000.0;
        let length = 0.8;
        for i in 0..11 {
            // set envelope params
            bell.envelopes[i].set_parameter(
                SynthParameterLabel::EnvelopeLevel,
                &SynthParameterValue::ScalarF32(bell.amps[i] * bell.main_level),
            );
            bell.envelopes[i].set_parameter(
                SynthParameterLabel::Release,
                &SynthParameterValue::ScalarF32(bell.durs[i] * length),
            );

            // set oscillator params
            bell.oscillators[i].set_parameter(
                SynthParameterLabel::PitchFrequency,
                &SynthParameterValue::ScalarF32(freq * bell.freqs[i] + bell.dets[i]),
            );
        }

        bell
    }
}

impl<const BUFSIZE: usize, const NCHAN: usize> Synth<BUFSIZE, NCHAN>
    for RissetBell<BUFSIZE, NCHAN>
{
    fn set_modulator(
        &mut self,
        par: SynthParameterLabel,
        init: f32,
        modulator: Modulator<BUFSIZE>,
    ) {
        self.lpf.set_modulator(par, init, modulator.clone());
        self.main_envelope
            .set_modulator(par, init, modulator.clone());
        self.balance.set_modulator(par, init, modulator);
    }

    fn set_parameter(&mut self, par: SynthParameterLabel, val: &SynthParameterValue) {
        self.lpf.set_parameter(par, val);
        self.main_envelope.set_parameter(par, val);
        self.balance.set_parameter(par, val);

        let mut update_internals = false;
        match par {
            SynthParameterLabel::ReverbMix => {
                if let SynthParameterValue::ScalarF32(r) = val {
                    self.reverb = *r
                }
            }
            SynthParameterLabel::DelayMix => {
                if let SynthParameterValue::ScalarF32(d) = val {
                    self.delay = *d
                }
            }
            SynthParameterLabel::PitchFrequency => {
                if let SynthParameterValue::ScalarF32(f) = val {
                    self.freq = *f
                };
                update_internals = true;
            }
            SynthParameterLabel::Attack => {
                if let SynthParameterValue::ScalarF32(f) = val {
                    self.atk = *f
                };
                update_internals = true;
            }
            SynthParameterLabel::Sustain => {
                if let SynthParameterValue::ScalarF32(s) = val {
                    self.sus = *s
                };
                update_internals = true;
            }
            SynthParameterLabel::Release => {
                if let SynthParameterValue::ScalarF32(r) = val {
                    self.rel = *r
                };
                update_internals = true;
            }
            SynthParameterLabel::EnvelopeLevel => {
                if let SynthParameterValue::ScalarF32(l) = val {
                    self.main_level = *l
                };
                update_internals = true;
            }
            _ => (),
        };

        if update_internals {
            let length = self.atk + self.sus + self.rel;
            for i in 0..11 {
                // set envelope params
                self.envelopes[i].set_parameter(
                    SynthParameterLabel::EnvelopeLevel,
                    &SynthParameterValue::ScalarF32(self.amps[i] * self.main_level),
                );
                self.envelopes[i].set_parameter(
                    SynthParameterLabel::Release,
                    &SynthParameterValue::ScalarF32(self.durs[i] * length),
                );

                // set oscillator params
                self.oscillators[i].set_parameter(
                    SynthParameterLabel::PitchFrequency,
                    &SynthParameterValue::ScalarF32(self.freq * self.freqs[i] + self.dets[i]),
                );
            }
        }
    }

    fn finish(&mut self) {
        self.main_envelope.finish();
    }

    fn is_finished(&self) -> bool {
        self.main_envelope.is_finished()
    }

    fn get_next_block(
        &mut self,
        start_sample: usize,
        sample_buffers: &[Vec<f32>],
    ) -> [[f32; BUFSIZE]; NCHAN] {
        // first osc
        let mut out: [f32; BUFSIZE] =
            self.oscillators[0].get_next_block(start_sample, sample_buffers);
        out = self.envelopes[0].process_block(out, start_sample, sample_buffers);

        // rest
        for i in 1..11 {
            let mut tmp_out: [f32; BUFSIZE] =
                self.oscillators[i].get_next_block(start_sample, sample_buffers);
            tmp_out = self.envelopes[i].process_block(tmp_out, start_sample, sample_buffers);

            for s in 0..BUFSIZE {
                out[s] += tmp_out[s];
            }
        }

        out = self.lpf.process_block(out, start_sample, sample_buffers);
        out = self
            .main_envelope
            .process_block(out, start_sample, sample_buffers);
        self.balance
            .process_block(out, start_sample, sample_buffers) // needs the additional info for the modulators
    }

    fn reverb_level(&self) -> f32 {
        self.reverb
    }

    fn delay_level(&self) -> f32 {
        self.delay
    }
}
