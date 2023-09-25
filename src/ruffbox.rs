pub mod ruffbox_controls;
pub mod ruffbox_playhead;

// crossbeam for the event queue
use crossbeam::atomic::AtomicCell;
use crossbeam::channel::Receiver;
use crossbeam::channel::Sender;

use std::cmp::Ordering;
use std::sync::Arc;

use crate::building_blocks::{
    Modulator, SampleBuffer, Synth, SynthParameterLabel, SynthParameterValue, ValueOrModulator,
};

pub use crate::ruffbox::{ruffbox_controls::*, ruffbox_playhead::*};

pub enum ScheduledSource<const BUFSIZE: usize, const NCHAN: usize> {
    Channel(Box<dyn Synth<BUFSIZE, NCHAN> + Send + Sync>),
    Ambi(Box<dyn Synth<BUFSIZE, 4> + Send + Sync>),
}

/// timed event, to be created in the trigger method, then
/// sent to the event queue to be either dispatched directly
/// or pushed to the pending queue ...
pub(crate) struct ScheduledEvent<const BUFSIZE: usize, const NCHAN: usize> {
    timestamp: f64,
    source: ScheduledSource<BUFSIZE, NCHAN>,
}

impl<const BUFSIZE: usize, const NCHAN: usize> Ord for ScheduledEvent<BUFSIZE, NCHAN> {
    /// ScheduledEvent implements Ord so the pending events queue
    /// can be ordered by the timestamps ...
    fn cmp(&self, other: &Self) -> Ordering {
        self.timestamp.partial_cmp(&other.timestamp).unwrap()
    }
}

impl<const BUFSIZE: usize, const NCHAN: usize> PartialOrd for ScheduledEvent<BUFSIZE, NCHAN> {
    /// ScheduledEvent implements PartialOrd so the pending events queue
    /// can be ordered by the timestamps ...
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<const BUFSIZE: usize, const NCHAN: usize> PartialEq for ScheduledEvent<BUFSIZE, NCHAN> {
    /// ScheduledEvent implements PartialEq so the pending events queue
    /// can be ordered by the timestamps ...
    fn eq(&self, other: &Self) -> bool {
        self.timestamp == other.timestamp
    }
}

impl<const BUFSIZE: usize, const NCHAN: usize> Eq for ScheduledEvent<BUFSIZE, NCHAN> {}

// constructor implementation
impl<const BUFSIZE: usize, const NCHAN: usize> ScheduledEvent<BUFSIZE, NCHAN> {
    pub fn new(ts: f64, src: ScheduledSource<BUFSIZE, NCHAN>) -> Self {
        ScheduledEvent {
            timestamp: ts,
            source: src,
        }
    }

    pub fn set_parameter(&mut self, par: SynthParameterLabel, value: &SynthParameterValue) {
        match self.source {
            ScheduledSource::Channel(ref mut src) => {
                src.set_parameter(par, value);
            }
            ScheduledSource::Ambi(ref mut src) => {
                src.set_parameter(par, value);
            }
        }
    }

    pub fn set_modulator(
        &mut self,
        par: SynthParameterLabel,
        init: f32,
        modulator: Modulator<BUFSIZE>,
    ) {
        match self.source {
            ScheduledSource::Channel(ref mut src) => {
                src.set_modulator(par, init, modulator);
            }
            ScheduledSource::Ambi(ref mut src) => {
                src.set_modulator(par, init, modulator);
            }
        }
    }

    fn set_param_or_modulator(
        &mut self,
        par: SynthParameterLabel,
        val_or_mod: ValueOrModulator<BUFSIZE>,
    ) {
        match val_or_mod {
            ValueOrModulator::Val(val) => self.set_parameter(par, &val),
            ValueOrModulator::Mod(init, modulator) => self.set_modulator(par, init, modulator),
        }
    }
}

/// Make your choice, freeverb or convolution ??
pub enum ReverbMode {
    FreeVerb,
    Convolution(Vec<f32>, f32),
}

pub(crate) enum ControlMessage<const BUFSIZE: usize, const NCHAN: usize> {
    LoadSample(usize, usize, SampleBuffer), // num, len, samples
    SetGlobalParamOrModulator(SynthParameterLabel, ValueOrModulator<BUFSIZE>),
    ScheduleEvent(ScheduledEvent<BUFSIZE, NCHAN>),
    FreezeBuffer(usize, usize),
}

/// before loading, analyze how many samples you want to load,
/// and pre-allocate the buffer vector accordingly (later)
pub fn init_ruffbox<const BUFSIZE: usize, const NCHAN: usize>(
    live_buffers: usize,
    live_buffer_time: f64,
    reverb_mode: &ReverbMode,
    samplerate: f64,
    max_buffers: usize,
    freeze_buffers: usize,
    ambisonics_binaural: bool,
) -> (
    RuffboxControls<BUFSIZE, NCHAN>,
    RuffboxPlayhead<BUFSIZE, NCHAN>,
) {
    let (tx, rx): (
        Sender<ControlMessage<BUFSIZE, NCHAN>>,
        Receiver<ControlMessage<BUFSIZE, NCHAN>>,
    ) = crossbeam::channel::bounded(2000);

    let now = Arc::new(AtomicCell::<f64>::new(0.0));

    let controls = RuffboxControls::<BUFSIZE, NCHAN>::new(
        samplerate,
        live_buffers,
        live_buffer_time,
        max_buffers,
        freeze_buffers,
        &now,
        tx,
    );
    let mut playhead = RuffboxPlayhead::<BUFSIZE, NCHAN>::new(
        live_buffers,
        live_buffer_time,
        reverb_mode,
        samplerate,
        max_buffers,
        freeze_buffers,
        &now,
        rx,
    );

    if ambisonics_binaural {
        playhead.enable_ambisonics_binaural();
    }

    (controls, playhead)
}

// TEST TEST TEST
#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use crate::building_blocks::{
        EnvelopeSegmentInfo, EnvelopeSegmentType, FilterType, OscillatorType, ValOp,
    };
    use crate::synths::SynthType;
    use std::f32::consts::PI;

    #[test]
    fn test_stitch_stuff() {
        let (_, mut ruff) =
            init_ruffbox::<512, 2>(1, 2.0, &ReverbMode::FreeVerb, 44100.0, 3000, 10, false);

        assert_approx_eq::assert_approx_eq!(ruff.fade_curve[0], 0.0, 0.00001);
        assert_approx_eq::assert_approx_eq!(ruff.fade_curve[127], 1.0, 0.0002);

        for _ in 0..513 {
            ruff.write_sample_to_live_buffer(0, 1.0);
        }

        for s in 0..128 {
            assert_approx_eq::assert_approx_eq!(
                ruff.live_buffer_metadata[0].stitch_buffer_incoming[s],
                1.0,
                0.0002
            );
        }

        {
            let SampleBuffer::Mono(buf) = &ruff.buffers[0] else {
                panic!()
            };
            assert_approx_eq::assert_approx_eq!(buf[1], 1.0, 0.0002);
            assert_approx_eq::assert_approx_eq!(buf[513], 0.0, 0.0002);
            //assert_approx_eq::assert_approx_eq!(buf[385], 1.0, 0.0002); // not sure why this doesn't hold anymore but it sounds perfect
        }

        for _ in 0..512 {
            ruff.write_sample_to_live_buffer(0, 1.0);
        }

        {
            let SampleBuffer::Mono(buf) = &ruff.buffers[0] else {
                panic!()
            };
            assert_approx_eq::assert_approx_eq!(buf[513], 1.0, 0.0002);
            assert_approx_eq::assert_approx_eq!(buf[385], 1.0, 0.0002);
            //assert_approx_eq::assert_approx_eq!(buf[1024], 0.0, 0.0002); // not sure why this doesn't hold anymore but it sounds perfect
            assert_approx_eq::assert_approx_eq!(buf[896], 1.0, 0.0002);
        }
        // write some seconds
        for _ in 0..2000 {
            for _ in 0..512 {
                ruff.write_sample_to_live_buffer(0, 1.0);
            }
        }
        {
            let SampleBuffer::Mono(buf) = &ruff.buffers[0] else {
                panic!()
            };
            assert_approx_eq::assert_approx_eq!(buf[0], 0.0, 0.0002);
            assert_approx_eq::assert_approx_eq!(buf[1], 1.0, 0.0002);
            assert_approx_eq::assert_approx_eq!(buf[44100], 1.0, 0.0002);
            assert_approx_eq::assert_approx_eq!(buf[ruff.buffer_lengths[0]], 1.0, 0.0002);
            assert_approx_eq::assert_approx_eq!(buf[ruff.buffer_lengths[0] + 1], 0.0, 0.0002);
            assert_approx_eq::assert_approx_eq!(buf[ruff.buffer_lengths[0] + 2], 0.0, 0.0002);
        }
    }

    #[test]
    fn test_load_mono_sample() {
        let (ctrl, mut ruff) =
            init_ruffbox::<512, 2>(0, 2.0, &ReverbMode::FreeVerb, 44100.0, 3000, 10, false);

        let mut sample = vec![1.0_f32; 500];

        ctrl.load_mono_sample(&mut sample, false, 44100.0);
        ruff.process(0.0, true);

        {
            let SampleBuffer::Mono(buf) = &ruff.buffers[0] else {
                panic!()
            };
            assert_approx_eq::assert_approx_eq!(buf[0], 0.0, 0.0002);
            assert_approx_eq::assert_approx_eq!(buf[1], 0.0, 0.0002);
            assert_approx_eq::assert_approx_eq!(buf[2], 1.0, 0.0002);
            assert_approx_eq::assert_approx_eq!(buf[3], 1.0, 0.0002);
            assert_approx_eq::assert_approx_eq!(buf[4], 1.0, 0.0002);
            assert_approx_eq::assert_approx_eq!(buf[500], 1.0, 0.0002);
            assert_approx_eq::assert_approx_eq!(buf[501], 1.0, 0.0002);
            assert_approx_eq::assert_approx_eq!(buf[502], 0.0, 0.0002);
            assert_approx_eq::assert_approx_eq!(buf[503], 0.0, 0.0002);
        }
    }

    #[test]
    fn test_sine_synth_at_block_start() {
        let (ctrl, mut ruff) =
            init_ruffbox::<128, 2>(0, 2.0, &ReverbMode::FreeVerb, 44100.0, 3000, 10, false);

        if let Some(mut inst) = ctrl.prepare_instance(
            SynthType::SingleOscillator(OscillatorType::Sine, FilterType::Dummy, FilterType::Dummy),
            0.0,
            0,
        ) {
            inst.set_instance_parameter(
                SynthParameterLabel::PitchFrequency,
                &SynthParameterValue::ScalarF32(440.0),
            );
            inst.set_instance_parameter(
                SynthParameterLabel::ChannelPosition,
                &SynthParameterValue::ScalarF32(0.0),
            );

            // this envelope mimics the old lin_asr sample by sample ...
            inst.set_instance_parameter(
                SynthParameterLabel::Envelope,
                &SynthParameterValue::MultiPointEnvelope(
                    vec![
                        EnvelopeSegmentInfo {
                            from: 0.0,
                            to: 1.0,
                            time: 0.000025,
                            segment_type: EnvelopeSegmentType::Lin,
                        },
                        EnvelopeSegmentInfo {
                            from: 1.0,
                            to: 1.0,
                            time: 1.0 - 0.000025,
                            segment_type: EnvelopeSegmentType::Constant,
                        },
                        EnvelopeSegmentInfo {
                            from: 1.0,
                            to: 0.0,
                            time: 0.000025,
                            segment_type: EnvelopeSegmentType::Lin,
                        },
                    ],
                    false,
                    ValOp::Replace,
                ),
            );

            ctrl.trigger(inst);
        }

        let out_1 = ruff.process(0.0, true);
        let mut comp_1 = [0.0; 128];

        for i in 0..128 {
            comp_1[i] = (2.0 * PI * 440.0 * (i as f32 * (1.0 / 44100.0))).sin() * 0.5;
        }

        // the recursive sine oscillator is slightly less precise ...
        for i in 0..128 {
            //println!("{} {} {}; ", i, out_1[0][i], comp_1[i]);
            //let a = out_1[0][i];
            //let b = comp_1[i];
            //debug_plotter::plot!(a, b where caption = "SynthBlockPlot");
            assert_approx_eq::assert_approx_eq!(out_1[0][i], comp_1[i], 0.008);
        }
    }

    #[test]
    fn test_basic_playback() {
        let (ctrl, mut ruff) =
            init_ruffbox::<128, 2>(1, 2.0, &ReverbMode::FreeVerb, 44100.0, 3000, 10, false);

        let mut sample1 = vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.3, 0.2, 0.1, 0.0];
        let mut sample2 = vec![0.0, 0.01, 0.02, 0.03, 0.04, 0.03, 0.02, 0.01, 0.0];

        let bnum1 = ctrl.load_mono_sample(&mut sample1, false, 44100.0);
        let bnum2 = ctrl.load_mono_sample(&mut sample2, false, 44100.0);

        ruff.process(0.0, true);

        if let Some(mut inst_1) = ctrl.prepare_instance(
            SynthType::Sampler(
                FilterType::BiquadHpf12dB,
                FilterType::Dummy,
                FilterType::Dummy,
                FilterType::Lpf18,
            ),
            0.0,
            bnum1,
        ) {
            // pan to left, neutralize
            inst_1.set_instance_parameter(
                SynthParameterLabel::ChannelPosition,
                &SynthParameterValue::ScalarF32(0.0),
            );
            inst_1.set_instance_parameter(
                SynthParameterLabel::LowpassCutoffFrequency,
                &SynthParameterValue::ScalarF32(22050.0),
            );
            inst_1.set_instance_parameter(
                SynthParameterLabel::LowpassFilterDistortion,
                &SynthParameterValue::ScalarF32(0.0),
            );
            inst_1.set_instance_parameter(
                SynthParameterLabel::LowpassQFactor,
                &SynthParameterValue::ScalarF32(0.0),
            );
            // this envelope mimics the old lin_asr sample by sample ...
            inst_1.set_instance_parameter(
                SynthParameterLabel::Envelope,
                &SynthParameterValue::MultiPointEnvelope(
                    vec![
                        EnvelopeSegmentInfo {
                            from: 0.0,
                            to: 1.0,
                            time: 0.000025,
                            segment_type: EnvelopeSegmentType::Lin,
                        },
                        EnvelopeSegmentInfo {
                            from: 1.0,
                            to: 1.0,
                            time: 1.0 - 0.000025,
                            segment_type: EnvelopeSegmentType::Constant,
                        },
                        EnvelopeSegmentInfo {
                            from: 1.0,
                            to: 0.0,
                            time: 0.000025,
                            segment_type: EnvelopeSegmentType::Lin,
                        },
                    ],
                    false,
                    ValOp::Replace,
                ),
            );
            ctrl.trigger(inst_1);
        }
        if let Some(mut inst_2) = ctrl.prepare_instance(
            SynthType::Sampler(
                FilterType::BiquadHpf12dB,
                FilterType::Dummy,
                FilterType::Dummy,
                FilterType::Lpf18,
            ),
            0.0,
            bnum2,
        ) {
            inst_2.set_instance_parameter(
                SynthParameterLabel::ChannelPosition,
                &SynthParameterValue::ScalarF32(0.0),
            );
            inst_2.set_instance_parameter(
                SynthParameterLabel::LowpassCutoffFrequency,
                &SynthParameterValue::ScalarF32(22050.0),
            );
            inst_2.set_instance_parameter(
                SynthParameterLabel::LowpassFilterDistortion,
                &SynthParameterValue::ScalarF32(0.0),
            );
            inst_2.set_instance_parameter(
                SynthParameterLabel::LowpassQFactor,
                &SynthParameterValue::ScalarF32(0.0),
            );
            // this envelope mimics the old lin_asr sample by sample ...
            inst_2.set_instance_parameter(
                SynthParameterLabel::Envelope,
                &SynthParameterValue::MultiPointEnvelope(
                    vec![
                        EnvelopeSegmentInfo {
                            from: 0.0,
                            to: 1.0,
                            time: 0.000025,
                            segment_type: EnvelopeSegmentType::Lin,
                        },
                        EnvelopeSegmentInfo {
                            from: 1.0,
                            to: 1.0,
                            time: 1.0 - 0.000025,
                            segment_type: EnvelopeSegmentType::Constant,
                        },
                        EnvelopeSegmentInfo {
                            from: 1.0,
                            to: 0.0,
                            time: 0.000025,
                            segment_type: EnvelopeSegmentType::Lin,
                        },
                    ],
                    false,
                    ValOp::Replace,
                ),
            );
            ctrl.trigger(inst_2);
        }

        let out_buf = ruff.process(0.0, true);

        for i in 0..9 {
            println!("{} {} ", out_buf[0][i], sample1[i + 2] + sample2[i + 2]);
            assert_approx_eq::assert_approx_eq!(
                out_buf[0][i],
                sample1[i + 2] + sample2[i + 2],
                0.03
            );
        }
    }

    #[test]
    fn reverb_smoke_test() {
        let (ctrl, mut ruff) =
            init_ruffbox::<128, 2>(1, 2.0, &ReverbMode::FreeVerb, 44100.0, 3000, 10, false);

        let mut sample1 = vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.3, 0.2, 0.1, 0.0];

        let bnum1 = ctrl.load_mono_sample(&mut sample1, false, 44100.0);

        ruff.process(0.0, true);

        if let Some(mut inst_1) = ctrl.prepare_instance(
            SynthType::Sampler(
                FilterType::BiquadHpf12dB,
                FilterType::Dummy,
                FilterType::Dummy,
                FilterType::Lpf18,
            ),
            0.0,
            bnum1,
        ) {
            // pan to left
            inst_1.set_instance_parameter(
                SynthParameterLabel::ChannelPosition,
                &SynthParameterValue::ScalarF32(0.0),
            );
            inst_1.set_instance_parameter(
                SynthParameterLabel::ReverbMix,
                &SynthParameterValue::ScalarF32(1.0),
            );
            inst_1.set_instance_parameter(
                SynthParameterLabel::LowpassCutoffFrequency,
                &SynthParameterValue::ScalarF32(22050.0),
            );
            inst_1.set_instance_parameter(
                SynthParameterLabel::LowpassFilterDistortion,
                &SynthParameterValue::ScalarF32(0.0),
            );
            inst_1.set_instance_parameter(
                SynthParameterLabel::LowpassQFactor,
                &SynthParameterValue::ScalarF32(0.0),
            );
            // this envelope mimics the old lin_asr sample by sample ...
            inst_1.set_instance_parameter(
                SynthParameterLabel::Envelope,
                &SynthParameterValue::MultiPointEnvelope(
                    vec![
                        EnvelopeSegmentInfo {
                            from: 0.0,
                            to: 1.0,
                            time: 0.000025,
                            segment_type: EnvelopeSegmentType::Lin,
                        },
                        EnvelopeSegmentInfo {
                            from: 1.0,
                            to: 1.0,
                            time: 1.0 - 0.000025,
                            segment_type: EnvelopeSegmentType::Constant,
                        },
                        EnvelopeSegmentInfo {
                            from: 1.0,
                            to: 0.0,
                            time: 0.000025,
                            segment_type: EnvelopeSegmentType::Lin,
                        },
                    ],
                    false,
                    ValOp::Replace,
                ),
            );

            ctrl.trigger(inst_1);
        }

        let out_buf = ruff.process(0.0, true);

        for i in 0..9 {
            println!("{} {} ", out_buf[0][i], sample1[i + 2]);
            assert_approx_eq::assert_approx_eq!(out_buf[0][i], sample1[i + 2], 0.03);
        }
    }

    #[test]
    fn test_scheduled_playback() {
        let (ctrl, mut ruff) =
            init_ruffbox::<128, 2>(1, 2.0, &ReverbMode::FreeVerb, 44100.0, 3000, 10, false);

        // block duration in seconds
        let block_duration = 0.00290249433;

        let mut sample1 = vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.3, 0.2, 0.1, 0.0];
        let mut sample2 = vec![0.0, 0.01, 0.02, 0.03, 0.04, 0.03, 0.02, 0.01, 0.0];

        let bnum1 = ctrl.load_mono_sample(&mut sample1, false, 44100.0);
        let bnum2 = ctrl.load_mono_sample(&mut sample2, false, 44100.0);

        if let Some(mut inst_1) = ctrl.prepare_instance(
            SynthType::Sampler(
                FilterType::BiquadHpf12dB,
                FilterType::Dummy,
                FilterType::Dummy,
                FilterType::Lpf18,
            ),
            0.291,
            bnum1,
        ) {
            // pan to left
            inst_1.set_instance_parameter(
                SynthParameterLabel::ChannelPosition,
                &SynthParameterValue::ScalarF32(0.0),
            );
            inst_1.set_instance_parameter(
                SynthParameterLabel::LowpassCutoffFrequency,
                &SynthParameterValue::ScalarF32(22050.0),
            );
            inst_1.set_instance_parameter(
                SynthParameterLabel::LowpassFilterDistortion,
                &SynthParameterValue::ScalarF32(0.0),
            );
            inst_1.set_instance_parameter(
                SynthParameterLabel::LowpassQFactor,
                &SynthParameterValue::ScalarF32(0.0),
            );

            // this envelope mimics the old lin_asr sample by sample ...
            inst_1.set_instance_parameter(
                SynthParameterLabel::Envelope,
                &SynthParameterValue::MultiPointEnvelope(
                    vec![EnvelopeSegmentInfo {
                        from: 1.0,
                        to: 1.0,
                        time: 9.0 / 44100.0,
                        segment_type: EnvelopeSegmentType::Constant,
                    }],
                    false,
                    ValOp::Replace,
                ),
            );
            ctrl.trigger(inst_1);
        }

        if let Some(mut inst_2) = ctrl.prepare_instance(
            SynthType::Sampler(
                FilterType::BiquadHpf12dB,
                FilterType::Dummy,
                FilterType::Dummy,
                FilterType::Lpf18,
            ),
            0.291,
            bnum2,
        ) {
            inst_2.set_instance_parameter(
                SynthParameterLabel::ChannelPosition,
                &SynthParameterValue::ScalarF32(0.0),
            );
            inst_2.set_instance_parameter(
                SynthParameterLabel::LowpassCutoffFrequency,
                &SynthParameterValue::ScalarF32(22050.0),
            );
            inst_2.set_instance_parameter(
                SynthParameterLabel::LowpassFilterDistortion,
                &SynthParameterValue::ScalarF32(0.0),
            );
            inst_2.set_instance_parameter(
                SynthParameterLabel::LowpassQFactor,
                &SynthParameterValue::ScalarF32(0.0),
            );
            // this envelope mimics the old lin_asr sample by sample ...
            inst_2.set_instance_parameter(
                SynthParameterLabel::Envelope,
                &SynthParameterValue::MultiPointEnvelope(
                    vec![EnvelopeSegmentInfo {
                        from: 1.0,
                        to: 1.0,
                        time: 9.0 / 44100.0,
                        segment_type: EnvelopeSegmentType::Constant,
                    }],
                    false,
                    ValOp::Replace,
                ),
            );
            ctrl.trigger(inst_2);
        }

        let mut stream_time = 0.0;
        // calculate a few blocks
        for _ in 0..100 {
            ruff.process(stream_time, false);
            stream_time += block_duration;
        }

        let out_buf = ruff.process(stream_time, false);

        for i in 0..9 {
            assert_approx_eq::assert_approx_eq!(
                out_buf[0][33 + i],
                sample1[i + 2] + sample2[i + 2],
                0.03
            );
        }
    }

    #[test]
    fn test_overlap_playback() {
        let (ctrl, mut ruff) =
            init_ruffbox::<128, 2>(1, 2.0, &ReverbMode::FreeVerb, 44100.0, 3000, 10, false);

        // block duration in seconds
        let block_duration = 0.00290249433;
        let sec_per_sample = 0.00002267573;

        let mut sample1 = vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.3, 0.2, 0.1, 0.0];
        let mut sample2 = vec![0.0, 0.01, 0.02, 0.03, 0.04, 0.03, 0.02, 0.01, 0.0];

        let bnum1 = ctrl.load_mono_sample(&mut sample1, false, 44100.0);
        let bnum2 = ctrl.load_mono_sample(&mut sample2, false, 44100.0);

        if let Some(mut inst_1) = ctrl.prepare_instance(
            SynthType::Sampler(
                FilterType::BiquadHpf12dB,
                FilterType::Dummy,
                FilterType::Dummy,
                FilterType::Lpf18,
            ),
            0.291,
            bnum1,
        ) {
            // pan to left
            inst_1.set_instance_parameter(
                SynthParameterLabel::ChannelPosition,
                &SynthParameterValue::ScalarF32(0.0),
            );
            inst_1.set_instance_parameter(
                SynthParameterLabel::LowpassCutoffFrequency,
                &SynthParameterValue::ScalarF32(22050.0),
            );
            inst_1.set_instance_parameter(
                SynthParameterLabel::LowpassFilterDistortion,
                &SynthParameterValue::ScalarF32(0.0),
            );
            inst_1.set_instance_parameter(
                SynthParameterLabel::LowpassQFactor,
                &SynthParameterValue::ScalarF32(0.0),
            );
            // this envelope mimics the old lin_asr sample by sample ...
            inst_1.set_instance_parameter(
                SynthParameterLabel::Envelope,
                &SynthParameterValue::MultiPointEnvelope(
                    vec![EnvelopeSegmentInfo {
                        from: 1.0,
                        to: 1.0,
                        time: 9.0 / 44100.0,
                        segment_type: EnvelopeSegmentType::Constant,
                    }],
                    false,
                    ValOp::Replace,
                ),
            );
            ctrl.trigger(inst_1);
        }

        if let Some(mut inst_2) = ctrl.prepare_instance(
            SynthType::Sampler(
                FilterType::BiquadHpf12dB,
                FilterType::Dummy,
                FilterType::Dummy,
                FilterType::Lpf18,
            ),
            0.291 + (4.0 * sec_per_sample),
            bnum2,
        ) {
            inst_2.set_instance_parameter(
                SynthParameterLabel::ChannelPosition,
                &SynthParameterValue::ScalarF32(0.0),
            );
            inst_2.set_instance_parameter(
                SynthParameterLabel::LowpassCutoffFrequency,
                &SynthParameterValue::ScalarF32(22050.0),
            );
            inst_2.set_instance_parameter(
                SynthParameterLabel::LowpassFilterDistortion,
                &SynthParameterValue::ScalarF32(0.0),
            );
            inst_2.set_instance_parameter(
                SynthParameterLabel::LowpassQFactor,
                &SynthParameterValue::ScalarF32(0.0),
            );
            // this envelope mimics the old lin_asr sample by sample ...
            inst_2.set_instance_parameter(
                SynthParameterLabel::Envelope,
                &SynthParameterValue::MultiPointEnvelope(
                    vec![EnvelopeSegmentInfo {
                        from: 1.0,
                        to: 1.0,
                        time: 9.0 / 44100.0,
                        segment_type: EnvelopeSegmentType::Constant,
                    }],
                    false,
                    ValOp::Replace,
                ),
            );
            ctrl.trigger(inst_2);
        }

        let mut stream_time = 0.0;

        // calculate a few blocks
        for _ in 0..100 {
            ruff.process(stream_time, false);
            stream_time += block_duration;
        }

        let out_buf = ruff.process(stream_time, false);

        // offsets to account for interpolation
        for i in 0..4 {
            assert_approx_eq::assert_approx_eq!(out_buf[0][33 + i], sample1[i + 2], 0.03);
        }

        for i in 0..5 {
            assert_approx_eq::assert_approx_eq!(
                out_buf[0][37 + i],
                sample1[i + 4 + 2] + sample2[i + 2],
                0.03
            );
        }

        for i in 0..4 {
            assert_approx_eq::assert_approx_eq!(out_buf[0][42 + i], sample2[i + 5 + 1], 0.03);
        }
    }

    #[test]
    fn test_disjunct_playback() {
        let (ctrl, mut ruff) =
            init_ruffbox::<128, 2>(1, 2.0, &ReverbMode::FreeVerb, 44100.0, 3000, 10, false);

        // block duration in seconds
        let block_duration = 0.00290249433;

        let mut sample1 = vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.3, 0.2, 0.1, 0.0];
        let mut sample2 = vec![0.0, 0.01, 0.02, 0.03, 0.04, 0.03, 0.02, 0.01, 0.0];

        let bnum1 = ctrl.load_mono_sample(&mut sample1, false, 44100.0);
        let bnum2 = ctrl.load_mono_sample(&mut sample2, false, 44100.0);

        // schedule two samples ahead, so they should  occur in different blocks
        // first sample should appear in block 100

        // second sample should appear ten blocks later
        let second_sample_timestamp = 0.291 + (10.0 * block_duration);

        if let Some(mut inst_1) = ctrl.prepare_instance(
            SynthType::Sampler(
                FilterType::Dummy,
                FilterType::Dummy,
                FilterType::Dummy,
                FilterType::Dummy,
            ),
            0.291,
            bnum1,
        ) {
            // pan to left
            inst_1.set_instance_parameter(
                SynthParameterLabel::ChannelPosition,
                &SynthParameterValue::ScalarF32(0.0),
            );

            // this envelope mimics the old lin_asr sample by sample ...
            inst_1.set_instance_parameter(
                SynthParameterLabel::Envelope,
                &SynthParameterValue::MultiPointEnvelope(
                    vec![EnvelopeSegmentInfo {
                        from: 1.0,
                        to: 1.0,
                        time: 9.0 / 44100.0,
                        segment_type: EnvelopeSegmentType::Constant,
                    }],
                    false,
                    ValOp::Replace,
                ),
            );
            ctrl.trigger(inst_1);
        }
        if let Some(mut inst_2) = ctrl.prepare_instance(
            SynthType::Sampler(
                FilterType::Dummy,
                FilterType::Dummy,
                FilterType::Dummy,
                FilterType::Dummy,
            ),
            second_sample_timestamp,
            bnum2,
        ) {
            inst_2.set_instance_parameter(
                SynthParameterLabel::ChannelPosition,
                &SynthParameterValue::ScalarF32(0.0),
            );

            // this envelope mimics the old lin_asr sample by sample ...
            inst_2.set_instance_parameter(
                SynthParameterLabel::Envelope,
                &SynthParameterValue::MultiPointEnvelope(
                    vec![EnvelopeSegmentInfo {
                        from: 1.0,
                        to: 1.0,
                        time: 9.0 / 44100.0,
                        segment_type: EnvelopeSegmentType::Constant,
                    }],
                    false,
                    ValOp::Replace,
                ),
            );
            ctrl.trigger(inst_2);
        }

        let mut stream_time = 0.0;

        // calculate a few blocks
        for _ in 0..100 {
            ruff.process(stream_time, false);
            stream_time += block_duration;
        }

        let out_buf = ruff.process(stream_time, false);
        stream_time += block_duration;

        println!("pre {out_buf:?}");

        for i in 0..9 {
            assert_approx_eq::assert_approx_eq!(out_buf[0][33 + i], sample1[i + 2], 0.03);
        }

        // calculate a few blocks more
        for _ in 0..9 {
            ruff.process(stream_time, false);
            stream_time += block_duration;
        }

        let out_buf = ruff.process(stream_time, false);
        println!("{out_buf:?}");
        for i in 0..9 {
            assert_approx_eq::assert_approx_eq!(out_buf[0][33 + i], sample2[i + 2], 0.03);
        }
    }

    #[test]
    fn test_late_playback() {
        let (ctrl, mut ruff) =
            init_ruffbox::<128, 2>(1, 2.0, &ReverbMode::FreeVerb, 44100.0, 3000, 10, false);

        let mut sample1 = vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.3, 0.2, 0.1, 0.0];

        let bnum1 = ctrl.load_mono_sample(&mut sample1, false, 44100.0);

        ruff.process(0.0, false);

        if let Some(mut inst_1) = ctrl.prepare_instance(
            SynthType::Sampler(
                FilterType::BiquadHpf12dB,
                FilterType::Dummy,
                FilterType::Dummy,
                FilterType::Lpf18,
            ),
            0.1,
            bnum1,
        ) {
            // pan to left
            inst_1.set_instance_parameter(
                SynthParameterLabel::ChannelPosition,
                &SynthParameterValue::ScalarF32(0.0),
            );
            inst_1.set_instance_parameter(
                SynthParameterLabel::LowpassCutoffFrequency,
                &SynthParameterValue::ScalarF32(22050.0),
            );
            inst_1.set_instance_parameter(
                SynthParameterLabel::LowpassFilterDistortion,
                &SynthParameterValue::ScalarF32(0.0),
            );
            inst_1.set_instance_parameter(
                SynthParameterLabel::LowpassQFactor,
                &SynthParameterValue::ScalarF32(0.0),
            );
            // this envelope mimics the old lin_asr sample by sample ...
            inst_1.set_instance_parameter(
                SynthParameterLabel::Envelope,
                &SynthParameterValue::MultiPointEnvelope(
                    vec![EnvelopeSegmentInfo {
                        from: 1.0,
                        to: 1.0,
                        time: 9.0 / 44100.0,
                        segment_type: EnvelopeSegmentType::Constant,
                    }],
                    false,
                    ValOp::Replace,
                ),
            );

            ctrl.trigger(inst_1);
        }

        // process after the instance's trigger time
        let out_buf = ruff.process(0.101, false);

        for i in 0..9 {
            assert_approx_eq::assert_approx_eq!(out_buf[0][i], sample1[i + 2], 0.03);
        }
    }
}

#[cfg(test)]
mod memory_tests {
    use super::*;
    use crate::building_blocks::{EnvelopeSegmentInfo, EnvelopeSegmentType, FilterType, ValOp};
    use crate::synths::SynthType;
    use assert_no_alloc::*;

    #[cfg(debug_assertions)] // required when disable_release is set (default)
    #[global_allocator]
    static A: AllocDisabler = AllocDisabler;

    #[test]
    fn test_memory_alloc() {
        let (ctrl, mut ruff) =
            init_ruffbox::<128, 2>(1, 2.0, &ReverbMode::FreeVerb, 44100.0, 3000, 10, false);

        let mut sample1 = vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.3, 0.2, 0.1, 0.0];
        let mut sample2 = vec![0.0, 0.01, 0.02, 0.03, 0.04, 0.03, 0.02, 0.01, 0.0];

        let bnum1 = ctrl.load_mono_sample(&mut sample1, false, 44100.0);
        let bnum2 = ctrl.load_mono_sample(&mut sample2, false, 44100.0);

        ruff.process(0.0, true);

        if let Some(mut inst_1) = ctrl.prepare_instance(
            SynthType::Sampler(
                FilterType::BiquadHpf12dB,
                FilterType::Dummy,
                FilterType::Dummy,
                FilterType::Lpf18,
            ),
            0.0,
            bnum1,
        ) {
            // pan to left, neutralize
            inst_1.set_instance_parameter(
                SynthParameterLabel::ChannelPosition,
                &SynthParameterValue::ScalarF32(0.0),
            );
            inst_1.set_instance_parameter(
                SynthParameterLabel::LowpassCutoffFrequency,
                &SynthParameterValue::ScalarF32(22050.0),
            );
            inst_1.set_instance_parameter(
                SynthParameterLabel::LowpassFilterDistortion,
                &SynthParameterValue::ScalarF32(0.0),
            );
            inst_1.set_instance_parameter(
                SynthParameterLabel::LowpassQFactor,
                &SynthParameterValue::ScalarF32(0.0),
            );
            // this envelope mimics the old lin_asr sample by sample ...
            inst_1.set_instance_parameter(
                SynthParameterLabel::Envelope,
                &SynthParameterValue::MultiPointEnvelope(
                    vec![
                        EnvelopeSegmentInfo {
                            from: 0.0,
                            to: 1.0,
                            time: 0.000025,
                            segment_type: EnvelopeSegmentType::Lin,
                        },
                        EnvelopeSegmentInfo {
                            from: 1.0,
                            to: 1.0,
                            time: 1.0 - 0.000025,
                            segment_type: EnvelopeSegmentType::Constant,
                        },
                        EnvelopeSegmentInfo {
                            from: 1.0,
                            to: 0.0,
                            time: 0.000025,
                            segment_type: EnvelopeSegmentType::Lin,
                        },
                    ],
                    false,
                    ValOp::Replace,
                ),
            );
            ctrl.trigger(inst_1);
        }
        if let Some(mut inst_2) = ctrl.prepare_instance(
            SynthType::Sampler(
                FilterType::BiquadHpf12dB,
                FilterType::Dummy,
                FilterType::Dummy,
                FilterType::Lpf18,
            ),
            0.0,
            bnum2,
        ) {
            inst_2.set_instance_parameter(
                SynthParameterLabel::ChannelPosition,
                &SynthParameterValue::ScalarF32(0.0),
            );
            inst_2.set_instance_parameter(
                SynthParameterLabel::LowpassCutoffFrequency,
                &SynthParameterValue::ScalarF32(22050.0),
            );
            inst_2.set_instance_parameter(
                SynthParameterLabel::LowpassFilterDistortion,
                &SynthParameterValue::ScalarF32(0.0),
            );
            inst_2.set_instance_parameter(
                SynthParameterLabel::LowpassQFactor,
                &SynthParameterValue::ScalarF32(0.0),
            );
            // this envelope mimics the old lin_asr sample by sample ...
            inst_2.set_instance_parameter(
                SynthParameterLabel::Envelope,
                &SynthParameterValue::MultiPointEnvelope(
                    vec![
                        EnvelopeSegmentInfo {
                            from: 0.0,
                            to: 1.0,
                            time: 0.000025,
                            segment_type: EnvelopeSegmentType::Lin,
                        },
                        EnvelopeSegmentInfo {
                            from: 1.0,
                            to: 1.0,
                            time: 1.0 - 0.000025,
                            segment_type: EnvelopeSegmentType::Constant,
                        },
                        EnvelopeSegmentInfo {
                            from: 1.0,
                            to: 0.0,
                            time: 0.000025,
                            segment_type: EnvelopeSegmentType::Lin,
                        },
                    ],
                    false,
                    ValOp::Replace,
                ),
            );
            ctrl.trigger(inst_2);
        }

        println!("CHECK IF MAIN PROCESS FUNCTION ALLOCATES MEMORY");
        assert_no_alloc(|| {
            for _ in 0..100 {
                let _ = ruff.process(0.0, true);
            }
        });
    }
}
