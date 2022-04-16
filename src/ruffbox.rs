pub mod ruffbox_controls;
pub mod ruffbox_playhead;

// crossbeam for the event queue
use crossbeam::atomic::AtomicCell;
use crossbeam::channel::Receiver;
use crossbeam::channel::Sender;

use std::cmp::Ordering;
use std::sync::Arc;

use crate::building_blocks::{Synth, SynthParameterLabel, SynthParameterValue};

pub use crate::ruffbox::{ruffbox_controls::*, ruffbox_playhead::*};

/// timed event, to be created in the trigger method, then
/// sent to the event queue to be either dispatched directly
/// or pushed to the pending queue ...
pub(crate) struct ScheduledEvent<const BUFSIZE: usize, const NCHAN: usize> {
    timestamp: f64,
    source: Box<dyn Synth<BUFSIZE, NCHAN> + Send + Sync>,
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
    pub fn new(ts: f64, src: Box<dyn Synth<BUFSIZE, NCHAN> + Send + Sync>) -> Self {
        ScheduledEvent {
            timestamp: ts,
            source: src,
        }
    }

    pub fn set_parameter(&mut self, par: SynthParameterLabel, value: &SynthParameterValue) {
        self.source.set_parameter(par, value);
    }
}

/// Make your choice, freeverb or convolution ??
pub enum ReverbMode {
    FreeVerb,
    Convolution(Vec<f32>, f32),
}

pub(crate) enum ControlMessage<const BUFSIZE: usize, const NCHAN: usize> {
    LoadSample(usize, usize, Vec<f32>), // num, len, samples
    SetGlobalParam(SynthParameterLabel, SynthParameterValue),
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
    let playhead = RuffboxPlayhead::<BUFSIZE, NCHAN>::new(
        live_buffers,
        live_buffer_time,
        reverb_mode,
        samplerate,
        max_buffers,
        freeze_buffers,
        &now,
        rx,
    );

    (controls, playhead)
}

// TEST TEST TEST
#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use crate::SourceType;
    use std::f32::consts::PI;

    #[test]
    fn test_stitch_stuff() {
        let (_, mut ruff) =
            init_ruffbox::<512, 2>(1, 2.0, &ReverbMode::FreeVerb, 44100.0, 3000, 10);

        assert_approx_eq::assert_approx_eq!(ruff.fade_curve[0], 0.0, 0.00001);
        assert_approx_eq::assert_approx_eq!(ruff.fade_curve[127], 1.0, 0.0002);

        for _ in 0..512 {
            ruff.write_sample_to_live_buffer(0, 1.0);
        }

        for s in 0..128 {
            assert_approx_eq::assert_approx_eq!(
                ruff.live_buffer_metadata[0].stitch_buffer[s],
                1.0,
                0.0002
            );
        }
        assert_approx_eq::assert_approx_eq!(ruff.buffers[0][1], 1.0, 0.0002);
        assert_approx_eq::assert_approx_eq!(ruff.buffers[0][513], 0.0, 0.0002);
        assert_approx_eq::assert_approx_eq!(ruff.buffers[0][385], 1.0, 0.0002);

        for _ in 0..512 {
            ruff.write_sample_to_live_buffer(0, 1.0);
        }

        assert_approx_eq::assert_approx_eq!(ruff.buffers[0][513], 1.0, 0.0002);
        assert_approx_eq::assert_approx_eq!(ruff.buffers[0][385], 1.0, 0.0002);
        assert_approx_eq::assert_approx_eq!(ruff.buffers[0][1024], 0.0, 0.0002);
        assert_approx_eq::assert_approx_eq!(ruff.buffers[0][896], 1.0, 0.0002);

        // write some seconds
        for _ in 0..2000 {
            for _ in 0..512 {
                ruff.write_sample_to_live_buffer(0, 1.0);
            }
        }

        assert_approx_eq::assert_approx_eq!(ruff.buffers[0][0], 0.0, 0.0002);
        assert_approx_eq::assert_approx_eq!(ruff.buffers[0][1], 1.0, 0.0002);
        assert_approx_eq::assert_approx_eq!(ruff.buffers[0][44100], 1.0, 0.0002);
        assert_approx_eq::assert_approx_eq!(ruff.buffers[0][ruff.buffer_lengths[0]], 1.0, 0.0002);
        assert_approx_eq::assert_approx_eq!(
            ruff.buffers[0][ruff.buffer_lengths[0] + 1],
            0.0,
            0.0002
        );
        assert_approx_eq::assert_approx_eq!(
            ruff.buffers[0][ruff.buffer_lengths[0] + 2],
            0.0,
            0.0002
        );
    }

    #[test]
    fn test_load_sample() {
        let (ctrl, mut ruff) =
            init_ruffbox::<512, 2>(0, 2.0, &ReverbMode::FreeVerb, 44100.0, 3000, 10);

        let mut sample = vec![1.0_f32; 500];

        ctrl.load_sample(&mut sample, false, 44100.0);
        ruff.process(0.0, true);

        assert_approx_eq::assert_approx_eq!(ruff.buffers[0][0], 0.0, 0.0002);
        assert_approx_eq::assert_approx_eq!(ruff.buffers[0][1], 1.0, 0.0002);
        assert_approx_eq::assert_approx_eq!(ruff.buffers[0][2], 1.0, 0.0002);
        assert_approx_eq::assert_approx_eq!(ruff.buffers[0][3], 1.0, 0.0002);
        assert_approx_eq::assert_approx_eq!(ruff.buffers[0][500], 1.0, 0.0002);
        assert_approx_eq::assert_approx_eq!(ruff.buffers[0][501], 0.0, 0.0002);
        assert_approx_eq::assert_approx_eq!(ruff.buffers[0][502], 0.0, 0.0002);
    }

    #[test]
    fn test_sine_synth_at_block_start() {
        let (ctrl, mut ruff) =
            init_ruffbox::<128, 2>(0, 2.0, &ReverbMode::FreeVerb, 44100.0, 3000, 10);

        if let Some(mut inst) = ctrl.prepare_instance(SourceType::SineSynth, 0.0, 0) {
            inst.set_instance_parameter(SynthParameterLabel::PitchFrequency, 440.0);
            inst.set_instance_parameter(SynthParameterLabel::ChannelPosition, 0.0);
            inst.set_instance_parameter(SynthParameterLabel::Level, 1.0);
            inst.set_instance_parameter(SynthParameterLabel::Attack, 0.0);
            inst.set_instance_parameter(SynthParameterLabel::Sustain, 1.0);
            inst.set_instance_parameter(SynthParameterLabel::Release, 0.0);

            ctrl.trigger(inst);
        }

        let out_1 = ruff.process(0.0, true);
        let mut comp_1 = [0.0; 128];

        for i in 0..128 {
            comp_1[i] = (2.0 * PI * 440.0 * (i as f32 * (1.0 / 44100.0))).sin()
        }

        for i in 0..128 {
            //println!("{} {} {}; ", i, out_1[0][i], comp_1[i]);
            assert_approx_eq::assert_approx_eq!(out_1[0][i], comp_1[i], 0.00001);
        }
    }

    #[test]
    fn test_basic_playback() {
        let (ctrl, mut ruff) =
            init_ruffbox::<128, 2>(1, 2.0, &ReverbMode::FreeVerb, 44100.0, 3000, 10);

        let mut sample1 = vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.3, 0.2, 0.1, 0.0];
        let mut sample2 = vec![0.0, 0.01, 0.02, 0.03, 0.04, 0.03, 0.02, 0.01, 0.0];

        let bnum1 = ctrl.load_sample(&mut sample1, false, 44100.0);
        let bnum2 = ctrl.load_sample(&mut sample2, false, 44100.0);

        ruff.process(0.0, true);

        if let Some(mut inst_1) = ctrl.prepare_instance(SourceType::Sampler, 0.0, bnum1) {
            // pan to left, neutralize
            inst_1.set_instance_parameter(SynthParameterLabel::ChannelPosition, 0.0);
            inst_1.set_instance_parameter(SynthParameterLabel::LowpassCutoffFrequency, 22050.0);
            inst_1.set_instance_parameter(SynthParameterLabel::LowpassFilterDistortion, 0.0);
            inst_1.set_instance_parameter(SynthParameterLabel::LowpassQFactor, 0.0);
            inst_1.set_instance_parameter(SynthParameterLabel::Attack, 0.0);
            inst_1.set_instance_parameter(SynthParameterLabel::Release, 0.0);
            inst_1.set_instance_parameter(SynthParameterLabel::Sustain, 9.0 / 44100.0);
            ctrl.trigger(inst_1);
        }
        if let Some(mut inst_2) = ctrl.prepare_instance(SourceType::Sampler, 0.0, bnum2) {
            inst_2.set_instance_parameter(SynthParameterLabel::ChannelPosition, 0.0);
            inst_2.set_instance_parameter(SynthParameterLabel::LowpassCutoffFrequency, 22050.0);
            inst_2.set_instance_parameter(SynthParameterLabel::LowpassFilterDistortion, 0.0);
            inst_2.set_instance_parameter(SynthParameterLabel::LowpassQFactor, 0.0);
            inst_2.set_instance_parameter(SynthParameterLabel::Attack, 0.0);
            inst_2.set_instance_parameter(SynthParameterLabel::Release, 0.0);
            inst_2.set_instance_parameter(SynthParameterLabel::Sustain, 9.0 / 44100.0);
            ctrl.trigger(inst_2);
        }

        let out_buf = ruff.process(0.0, true);

        for i in 0..9 {
            println!("{} {} ", out_buf[0][i], sample1[i + 1] + sample2[i + 1]);
            assert_approx_eq::assert_approx_eq!(
                out_buf[0][i],
                sample1[i + 1] + sample2[i + 1],
                0.03
            );
        }
    }

    #[test]
    fn reverb_smoke_test() {
        let (ctrl, mut ruff) =
            init_ruffbox::<128, 2>(1, 2.0, &ReverbMode::FreeVerb, 44100.0, 3000, 10);

        let mut sample1 = vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.3, 0.2, 0.1, 0.0];

        let bnum1 = ctrl.load_sample(&mut sample1, false, 44100.0);

        ruff.process(0.0, true);

        if let Some(mut inst_1) = ctrl.prepare_instance(SourceType::Sampler, 0.0, bnum1) {
            // pan to left
            inst_1.set_instance_parameter(SynthParameterLabel::ChannelPosition, 0.0);
            inst_1.set_instance_parameter(SynthParameterLabel::ReverbMix, 1.0);
            inst_1.set_instance_parameter(SynthParameterLabel::LowpassCutoffFrequency, 22050.0);
            inst_1.set_instance_parameter(SynthParameterLabel::LowpassFilterDistortion, 0.0);
            inst_1.set_instance_parameter(SynthParameterLabel::LowpassQFactor, 0.0);
            inst_1.set_instance_parameter(SynthParameterLabel::Attack, 0.0);
            inst_1.set_instance_parameter(SynthParameterLabel::Release, 0.0);
            inst_1.set_instance_parameter(SynthParameterLabel::Sustain, 9.0 / 44100.0);

            ctrl.trigger(inst_1);
        }

        let out_buf = ruff.process(0.0, true);

        for i in 0..9 {
            println!("{} {} ", out_buf[0][i], sample1[i + 1]);
            assert_approx_eq::assert_approx_eq!(out_buf[0][i], sample1[i + 1], 0.03);
        }
    }

    #[test]
    fn test_scheduled_playback() {
        let (ctrl, mut ruff) =
            init_ruffbox::<128, 2>(1, 2.0, &ReverbMode::FreeVerb, 44100.0, 3000, 10);

        // block duration in seconds
        let block_duration = 0.00290249433;

        let mut sample1 = vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.3, 0.2, 0.1, 0.0];
        let mut sample2 = vec![0.0, 0.01, 0.02, 0.03, 0.04, 0.03, 0.02, 0.01, 0.0];

        let bnum1 = ctrl.load_sample(&mut sample1, false, 44100.0);
        let bnum2 = ctrl.load_sample(&mut sample2, false, 44100.0);

        if let Some(mut inst_1) = ctrl.prepare_instance(SourceType::Sampler, 0.291, bnum1) {
            // pan to left
            inst_1.set_instance_parameter(SynthParameterLabel::ChannelPosition, 0.0);
            inst_1.set_instance_parameter(SynthParameterLabel::LowpassCutoffFrequency, 22050.0);
            inst_1.set_instance_parameter(SynthParameterLabel::LowpassFilterDistortion, 0.0);
            inst_1.set_instance_parameter(SynthParameterLabel::LowpassQFactor, 0.0);
            inst_1.set_instance_parameter(SynthParameterLabel::Attack, 0.0);
            inst_1.set_instance_parameter(SynthParameterLabel::Release, 0.0);
            inst_1.set_instance_parameter(SynthParameterLabel::Sustain, 9.0 / 44100.0);
            ctrl.trigger(inst_1);
        }

        if let Some(mut inst_2) = ctrl.prepare_instance(SourceType::Sampler, 0.291, bnum2) {
            inst_2.set_instance_parameter(SynthParameterLabel::ChannelPosition, 0.0);
            inst_2.set_instance_parameter(SynthParameterLabel::LowpassCutoffFrequency, 22050.0);
            inst_2.set_instance_parameter(SynthParameterLabel::LowpassFilterDistortion, 0.0);
            inst_2.set_instance_parameter(SynthParameterLabel::LowpassQFactor, 0.0);
            inst_2.set_instance_parameter(SynthParameterLabel::Attack, 0.0);
            inst_2.set_instance_parameter(SynthParameterLabel::Release, 0.0);
            inst_2.set_instance_parameter(SynthParameterLabel::Sustain, 9.0 / 44100.0);
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
                sample1[i + 1] + sample2[i + 1],
                0.03
            );
        }
    }

    #[test]
    fn test_overlap_playback() {
        let (ctrl, mut ruff) =
            init_ruffbox::<128, 2>(1, 2.0, &ReverbMode::FreeVerb, 44100.0, 3000, 10);

        // block duration in seconds
        let block_duration = 0.00290249433;
        let sec_per_sample = 0.00002267573;

        let mut sample1 = vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.3, 0.2, 0.1, 0.0];
        let mut sample2 = vec![0.0, 0.01, 0.02, 0.03, 0.04, 0.03, 0.02, 0.01, 0.0];

        let bnum1 = ctrl.load_sample(&mut sample1, false, 44100.0);
        let bnum2 = ctrl.load_sample(&mut sample2, false, 44100.0);

        if let Some(mut inst_1) = ctrl.prepare_instance(SourceType::Sampler, 0.291, bnum1) {
            // pan to left
            inst_1.set_instance_parameter(SynthParameterLabel::ChannelPosition, 0.0);
            inst_1.set_instance_parameter(SynthParameterLabel::LowpassCutoffFrequency, 22050.0);
            inst_1.set_instance_parameter(SynthParameterLabel::LowpassFilterDistortion, 0.0);
            inst_1.set_instance_parameter(SynthParameterLabel::LowpassQFactor, 0.0);
            inst_1.set_instance_parameter(SynthParameterLabel::Attack, 0.0);
            inst_1.set_instance_parameter(SynthParameterLabel::Release, 0.0);
            inst_1.set_instance_parameter(SynthParameterLabel::Sustain, 9.0 / 44100.0);
            ctrl.trigger(inst_1);
        }

        if let Some(mut inst_2) =
            ctrl.prepare_instance(SourceType::Sampler, 0.291 + (4.0 * sec_per_sample), bnum2)
        {
            inst_2.set_instance_parameter(SynthParameterLabel::ChannelPosition, 0.0);
            inst_2.set_instance_parameter(SynthParameterLabel::LowpassCutoffFrequency, 22050.0);
            inst_2.set_instance_parameter(SynthParameterLabel::LowpassFilterDistortion, 0.0);
            inst_2.set_instance_parameter(SynthParameterLabel::LowpassQFactor, 0.0);
            inst_2.set_instance_parameter(SynthParameterLabel::Attack, 0.0);
            inst_2.set_instance_parameter(SynthParameterLabel::Release, 0.0);
            inst_2.set_instance_parameter(SynthParameterLabel::Sustain, 9.0 / 44100.0);
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
            assert_approx_eq::assert_approx_eq!(out_buf[0][33 + i], sample1[i + 1], 0.03);
        }

        for i in 0..5 {
            assert_approx_eq::assert_approx_eq!(
                out_buf[0][37 + i],
                sample1[i + 4 + 1] + sample2[i + 1],
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
            init_ruffbox::<128, 2>(1, 2.0, &ReverbMode::FreeVerb, 44100.0, 3000, 10);

        // block duration in seconds
        let block_duration = 0.00290249433;

        let mut sample1 = vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.3, 0.2, 0.1, 0.0];
        let mut sample2 = vec![0.0, 0.01, 0.02, 0.03, 0.04, 0.03, 0.02, 0.01, 0.0];

        let bnum1 = ctrl.load_sample(&mut sample1, false, 44100.0);
        let bnum2 = ctrl.load_sample(&mut sample2, false, 44100.0);

        // schedule two samples ahead, so they should  occur in different blocks
        // first sample should appear in block 100

        // second sample should appear ten blocks later
        let second_sample_timestamp = 0.291 + (10.0 * block_duration);

        if let Some(mut inst_1) = ctrl.prepare_instance(SourceType::Sampler, 0.291, bnum1) {
            // pan to left
            inst_1.set_instance_parameter(SynthParameterLabel::ChannelPosition, 0.0);
            inst_1.set_instance_parameter(SynthParameterLabel::LowpassCutoffFrequency, 22050.0);
            inst_1.set_instance_parameter(SynthParameterLabel::LowpassFilterDistortion, 0.0);
            inst_1.set_instance_parameter(SynthParameterLabel::LowpassQFactor, 0.0);
            inst_1.set_instance_parameter(SynthParameterLabel::Attack, 0.0);
            inst_1.set_instance_parameter(SynthParameterLabel::Release, 0.0);
            inst_1.set_instance_parameter(SynthParameterLabel::Sustain, 9.0 / 44100.0);
            ctrl.trigger(inst_1);
        }
        if let Some(mut inst_2) =
            ctrl.prepare_instance(SourceType::Sampler, second_sample_timestamp, bnum2)
        {
            inst_2.set_instance_parameter(SynthParameterLabel::ChannelPosition, 0.0);
            inst_2.set_instance_parameter(SynthParameterLabel::LowpassCutoffFrequency, 22050.0);
            inst_2.set_instance_parameter(SynthParameterLabel::LowpassFilterDistortion, 0.0);
            inst_2.set_instance_parameter(SynthParameterLabel::LowpassQFactor, 0.0);
            inst_2.set_instance_parameter(SynthParameterLabel::Attack, 0.0);
            inst_2.set_instance_parameter(SynthParameterLabel::Release, 0.0);
            inst_2.set_instance_parameter(SynthParameterLabel::Sustain, 9.0 / 44100.0);
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

        for i in 0..9 {
            assert_approx_eq::assert_approx_eq!(out_buf[0][33 + i], sample1[i + 1], 0.03);
        }

        // calculate a few blocks more
        for _ in 0..9 {
            ruff.process(stream_time, false);
            stream_time += block_duration;
        }

        let out_buf = ruff.process(stream_time, false);

        for i in 0..9 {
            assert_approx_eq::assert_approx_eq!(out_buf[0][33 + i], sample2[i + 1], 0.03);
        }
    }

    #[test]
    fn test_late_playback() {
        let (ctrl, mut ruff) =
            init_ruffbox::<128, 2>(1, 2.0, &ReverbMode::FreeVerb, 44100.0, 3000, 10);

        let mut sample1 = vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.3, 0.2, 0.1, 0.0];

        let bnum1 = ctrl.load_sample(&mut sample1, false, 44100.0);

        ruff.process(0.0, false);

        if let Some(mut inst_1) = ctrl.prepare_instance(SourceType::Sampler, 0.1, bnum1) {
            // pan to left
            inst_1.set_instance_parameter(SynthParameterLabel::ChannelPosition, 0.0);
            inst_1.set_instance_parameter(SynthParameterLabel::LowpassCutoffFrequency, 22050.0);
            inst_1.set_instance_parameter(SynthParameterLabel::LowpassFilterDistortion, 0.0);
            inst_1.set_instance_parameter(SynthParameterLabel::LowpassQFactor, 0.0);
            inst_1.set_instance_parameter(SynthParameterLabel::Attack, 0.0);
            inst_1.set_instance_parameter(SynthParameterLabel::Release, 0.0);
            inst_1.set_instance_parameter(SynthParameterLabel::Sustain, 9.0 / 44100.0);

            ctrl.trigger(inst_1);
        }

        // process after the instance's trigger time
        let out_buf = ruff.process(0.101, false);

        for i in 0..9 {
            assert_approx_eq::assert_approx_eq!(out_buf[0][i], sample1[i + 1], 0.03);
        }
    }
}
