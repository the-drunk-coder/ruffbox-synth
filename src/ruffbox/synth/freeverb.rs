use crate::ruffbox::synth::SynthParameter;

struct FreeverbDefaultTuning;

/// The default tuning, as it is found in the original freeverb code.
impl FreeverbDefaultTuning {
    const FIXED_GAIN: f32 = 0.015;
    const SCALE_WET: f32 = 3.0;
    const SCALE_DAMP: f32 = 0.4;
    const SCALE_ROOM: f32 = 0.28;
    const OFFSET_ROOM: f32 = 0.7;
    const INITIAL_ROOM: f32 = 0.5;
    const INITIAL_DAMP: f32 = 0.5;
    const INITIAL_WET: f32 = 1.0 / FreeverbDefaultTuning::SCALE_WET; // scalewet
    const INITIAL_WIDTH: f32 = 1.0;
    const STEREO_SPREAD: usize = 23;

    // NOTE FROM ORIGIAL CODE:
    // These values assume 44.1KHz sample rate
    // they will probably be OK for 48KHz sample rate
    // but would need scaling for 96KHz (or other) sample rates.
    // The values were obtained by listening tests.

    const COMB_TUNING_L1: usize = 1116;
    const COMB_TUNING_R1: usize = 1116 + FreeverbDefaultTuning::STEREO_SPREAD;
    const COMB_TUNING_L2: usize = 1188;
    const COMB_TUNING_R2: usize = 1188 + FreeverbDefaultTuning::STEREO_SPREAD;
    const COMB_TUNING_L3: usize = 1277;
    const COMB_TUNING_R3: usize = 1277 + FreeverbDefaultTuning::STEREO_SPREAD;
    const COMB_TUNING_L4: usize = 1356;
    const COMB_TUNING_R4: usize = 1356 + FreeverbDefaultTuning::STEREO_SPREAD;
    const COMB_TUNING_L5: usize = 1422;
    const COMB_TUNING_R5: usize = 1422 + FreeverbDefaultTuning::STEREO_SPREAD;
    const COMB_TUNING_L6: usize = 1491;
    const COMB_TUNING_R6: usize = 1491 + FreeverbDefaultTuning::STEREO_SPREAD;
    const COMB_TUNING_L7: usize = 1557;
    const COMB_TUNING_R7: usize = 1557 + FreeverbDefaultTuning::STEREO_SPREAD;
    const COMB_TUNING_L8: usize = 1617;
    const COMB_TUNING_R8: usize = 1617 + FreeverbDefaultTuning::STEREO_SPREAD;
    const ALLPASS_TUNING_L1: usize = 556;
    const ALLPASS_TUNING_R1: usize = 556 + FreeverbDefaultTuning::STEREO_SPREAD;
    const ALLPASS_TUNING_L2: usize = 441;
    const ALLPASS_TUNING_R2: usize = 441 + FreeverbDefaultTuning::STEREO_SPREAD;
    const ALLPASS_TUNING_L3: usize = 341;
    const ALLPASS_TUNING_R3: usize = 341 + FreeverbDefaultTuning::STEREO_SPREAD;
    const ALLPASS_TUNING_L4: usize = 225;
    const ALLPASS_TUNING_R4: usize = 225 + FreeverbDefaultTuning::STEREO_SPREAD;
}

/**
 * A sample-wise allpass filter to be used with the reverb effect.
 *
 * Adapted from the original Freeverb allpass filter.
 */
struct Allpass {
    delay_buffer: Vec<f32>,
    delay_buffer_size: usize,
    delay_idx: usize,
    feedback: f32,
}

impl Allpass {
    pub fn with_buffer_size(buf_size: usize) -> Self {
        Allpass {
            delay_buffer: vec![0.0; buf_size],
            delay_buffer_size: buf_size,
            delay_idx: 0,
            feedback: 0.5,
        }
    }

    #[inline(always)]
    pub fn process_sample(&mut self, sample: f32) -> f32 {
        let mut buf_out: f32 = self.delay_buffer[self.delay_idx];
        if !buf_out.is_normal() {
            buf_out = 0.0;
        }

        let out = (-1.0 * sample) + buf_out;
        self.delay_buffer[self.delay_idx] = sample + (buf_out * self.feedback);

        // increment delay idx
        self.delay_idx += 1;
        if self.delay_idx >= self.delay_buffer_size {
            self.delay_idx = 0;
        }

        out
    }
}

/**
 * A sample-wise comb filter to be used with the reverb effect.
 *
 * Adapted from the original Freeverb comb filter.
 */
struct Comb {
    delay_buffer: Vec<f32>,
    delay_buffer_size: usize,
    delay_idx: usize,
    feedback: f32,
    filterstore: f32,
    damp1: f32,
    damp2: f32,
}

impl Comb {
    pub fn with_buffer_size(buf_size: usize) -> Self {
        Comb {
            delay_buffer: vec![0.0; buf_size],
            delay_buffer_size: buf_size,
            delay_idx: 0,
            feedback: 0.5,
            filterstore: 0.0,
            damp1: 0.5,
            damp2: 0.5,
        }
    }

    #[inline(always)]
    pub fn process_sample(&mut self, sample: f32) -> f32 {
        let mut out = self.delay_buffer[self.delay_idx];
        if !out.is_normal() {
            out = 0.0;
        }

        self.filterstore = (out * self.damp2) + (self.filterstore * self.damp1);
        if !self.filterstore.is_normal() {
            self.filterstore = 0.0;
        }

        self.delay_buffer[self.delay_idx] = sample + (self.filterstore * self.feedback);

        // increment delay idx
        self.delay_idx += 1;
        if self.delay_idx >= self.delay_buffer_size {
            self.delay_idx = 0;
        }

        out
    }
}

/**
 * Multichannel extension of the classic Schroeder/Moorer reverb,
 * adapted from the original Freeverb code written by "Jezar at Dreampoint".
 *
 * It's slightly simplified and doesn't have a dry/wet paramater,
 * as it's meant to be used on a bus rather than directly on a track.
 * For the same reason, it only has a replacing process function.
 *
 * The freeze mode feature has been left out as well.
 *
 * The multichannel expansion is somewhat experimental and might change
 */
pub struct MultichannelFreeverb<const BUFSIZE: usize, const NCHAN: usize> {
    combs: Vec<Vec<(Comb, Comb)>>,
    allpasses: Vec<Vec<(Allpass, Allpass)>>,
    gain: f32,
    roomsize: f32,
    damp: f32,
    wet: f32,
    wet1: f32,
    wet2: f32,
    width: f32,
}

impl<const BUFSIZE: usize, const NCHAN: usize> MultichannelFreeverb<BUFSIZE, NCHAN> {
    pub fn new() -> Self {
        let mut combs = Vec::new();
        let mut allpasses = Vec::new();
        for i in 0..NCHAN {
            combs.push(Vec::new());
            allpasses.push(Vec::new());

            combs[i].push((
                Comb::with_buffer_size(FreeverbDefaultTuning::COMB_TUNING_L1),
                Comb::with_buffer_size(FreeverbDefaultTuning::COMB_TUNING_R1),
            ));
            combs[i].push((
                Comb::with_buffer_size(FreeverbDefaultTuning::COMB_TUNING_L2),
                Comb::with_buffer_size(FreeverbDefaultTuning::COMB_TUNING_R2),
            ));
            combs[i].push((
                Comb::with_buffer_size(FreeverbDefaultTuning::COMB_TUNING_L3),
                Comb::with_buffer_size(FreeverbDefaultTuning::COMB_TUNING_R3),
            ));
            combs[i].push((
                Comb::with_buffer_size(FreeverbDefaultTuning::COMB_TUNING_L4),
                Comb::with_buffer_size(FreeverbDefaultTuning::COMB_TUNING_R4),
            ));
            combs[i].push((
                Comb::with_buffer_size(FreeverbDefaultTuning::COMB_TUNING_L5),
                Comb::with_buffer_size(FreeverbDefaultTuning::COMB_TUNING_R5),
            ));
            combs[i].push((
                Comb::with_buffer_size(FreeverbDefaultTuning::COMB_TUNING_L6),
                Comb::with_buffer_size(FreeverbDefaultTuning::COMB_TUNING_R6),
            ));
            combs[i].push((
                Comb::with_buffer_size(FreeverbDefaultTuning::COMB_TUNING_L7),
                Comb::with_buffer_size(FreeverbDefaultTuning::COMB_TUNING_R7),
            ));
            combs[i].push((
                Comb::with_buffer_size(FreeverbDefaultTuning::COMB_TUNING_L8),
                Comb::with_buffer_size(FreeverbDefaultTuning::COMB_TUNING_R8),
            ));

            allpasses[i].push((
                Allpass::with_buffer_size(FreeverbDefaultTuning::ALLPASS_TUNING_L1),
                Allpass::with_buffer_size(FreeverbDefaultTuning::ALLPASS_TUNING_R1),
            ));
            allpasses[i].push((
                Allpass::with_buffer_size(FreeverbDefaultTuning::ALLPASS_TUNING_L2),
                Allpass::with_buffer_size(FreeverbDefaultTuning::ALLPASS_TUNING_R2),
            ));
            allpasses[i].push((
                Allpass::with_buffer_size(FreeverbDefaultTuning::ALLPASS_TUNING_L3),
                Allpass::with_buffer_size(FreeverbDefaultTuning::ALLPASS_TUNING_R3),
            ));
            allpasses[i].push((
                Allpass::with_buffer_size(FreeverbDefaultTuning::ALLPASS_TUNING_L4),
                Allpass::with_buffer_size(FreeverbDefaultTuning::ALLPASS_TUNING_R4),
            ));
        }

        let wet = FreeverbDefaultTuning::INITIAL_WET * FreeverbDefaultTuning::SCALE_WET;
        let wet1 = wet * ((FreeverbDefaultTuning::INITIAL_WIDTH / 2.0) + 0.5);
        let wet2 = wet * ((1.0 - FreeverbDefaultTuning::INITIAL_WIDTH) / 2.0);

        MultichannelFreeverb {
            combs: combs,
            allpasses: allpasses,
            gain: FreeverbDefaultTuning::FIXED_GAIN,
            roomsize: FreeverbDefaultTuning::INITIAL_ROOM,
            damp: FreeverbDefaultTuning::INITIAL_DAMP,
            wet: FreeverbDefaultTuning::INITIAL_WET,
            wet1: wet1,
            wet2: wet2,
            width: FreeverbDefaultTuning::INITIAL_WIDTH,
        }
    }

    pub fn set_roomsize(&mut self, value: f32) {
        self.roomsize =
            (value * FreeverbDefaultTuning::SCALE_ROOM) + FreeverbDefaultTuning::OFFSET_ROOM;
        // accumulate comb filters in parallel
        for comb in self.combs.iter_mut() {
            for (c_l, c_r) in comb.iter_mut() {
                c_l.feedback = self.roomsize;
                c_r.feedback = self.roomsize;
            }
        }
    }

    pub fn set_damp(&mut self, value: f32) {
        self.damp = value * FreeverbDefaultTuning::SCALE_DAMP;
        for comb in self.combs.iter_mut() {
            for (c_l, c_r) in comb.iter_mut() {
                c_l.damp1 = self.damp;
                c_l.damp2 = 1.0 - self.damp;
                c_r.damp1 = self.damp;
                c_r.damp2 = 1.0 - self.damp;
            }
        }
    }

    pub fn set_wet(&mut self, value: f32) {
        self.wet = value * FreeverbDefaultTuning::SCALE_WET;
        self.wet1 = self.wet * ((self.width / 2.0) + 0.5);
        self.wet2 = self.wet * ((1.0 - self.width) / 2.0);
    }

    #[allow(dead_code)]
    pub fn set_width(&mut self, value: f32) {
        self.width = value;
        self.set_wet(self.width);
    }

    pub fn set_parameter(&mut self, par: SynthParameter, val: f32) {
        match par {
            SynthParameter::ReverbRoomsize => self.set_roomsize(val),
            SynthParameter::ReverbDampening => self.set_damp(val),
            _ => (),
        };
    }

    /**
     * Main processing routine.
     * Takes a mono block, as this would be downmixed anyway.
     */
    pub fn process(&mut self, block: [[f32; BUFSIZE]; NCHAN]) -> [[f32; BUFSIZE]; NCHAN] {
        let mut out_buf = [[0.0; BUFSIZE]; NCHAN];
        let cur_gain = self.gain * 0.5;

        for c in 0..NCHAN {
            let upper = (c + 1) % NCHAN;
            for i in 0..BUFSIZE {
                let mut out_l = 0.0;
                let mut out_r = 0.0;

                let in_mix = (block[c][i] + block[upper][i]) * cur_gain;

                // accumulate comb filters in parallel
                for (comb_l, comb_r) in self.combs[c].iter_mut() {
                    out_l += comb_l.process_sample(in_mix);
                    out_r += comb_r.process_sample(in_mix);
                }

                // accumulate allpass filters in series
                for (allpass_l, allpass_r) in self.allpasses[c].iter_mut() {
                    out_l = allpass_l.process_sample(out_l);
                    out_r = allpass_r.process_sample(out_r);
                }

                out_buf[c][i] = (out_l * self.wet1) + (out_r * self.wet2);
                out_buf[upper][i] = (out_r * self.wet1) + (out_l * self.wet2);
            }
        }

        out_buf
    }
}
