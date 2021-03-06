pub mod synth;

// crossbeam for the event queue
use crossbeam::atomic::AtomicCell;
use crossbeam::channel::Receiver;
use crossbeam::channel::Sender;

use std::collections::HashMap;

use std::cmp::Ordering;

use crate::ruffbox::synth::delay::MultichannelDelay;
use crate::ruffbox::synth::freeverb::MultichannelFreeverb;
use crate::ruffbox::synth::synths::*;
use crate::ruffbox::synth::SourceType;
use crate::ruffbox::synth::Synth;
use crate::ruffbox::synth::SynthParameter;

/// timed event, to be created in the trigger method, then
/// sent to the event queue to be either dispatched directly
/// or pushed to the pending queue ...
struct ScheduledEvent<const BUFSIZE: usize, const NCHAN: usize> {
    timestamp: f64,
    source: Box<dyn Synth<BUFSIZE, NCHAN> + Send>,
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
    pub fn new(ts: f64, src: Box<dyn Synth<BUFSIZE, NCHAN> + Send>) -> Self {
        ScheduledEvent {
            timestamp: ts,
            source: src,
        }
    }

    pub fn set_parameter(&mut self, par: SynthParameter, value: f32) {
        self.source.set_parameter(par, value);
    }
}

/// the main synth instance
pub struct Ruffbox<const BUFSIZE: usize, const NCHAN: usize> {
    running_instances: Vec<Box<dyn Synth<BUFSIZE, NCHAN> + Send>>,
    pending_events: Vec<ScheduledEvent<BUFSIZE, NCHAN>>,
    buffers: Vec<Vec<f32>>,
    buffer_lengths: Vec<usize>,
    live_buffer_idx: usize,
    live_buffer_current_block: usize,
    live_buffer_stitch_size: usize,
    non_stitch_size: usize,
    fade_stitch_idx: usize,
    fade_curve: Vec<f32>,
    stitch_buffer: Vec<f32>,
    bufsize: usize,
    prepared_instance_map: HashMap<usize, ScheduledEvent<BUFSIZE, NCHAN>>,
    instance_counter: AtomicCell<usize>,
    new_instances_q_send: crossbeam::channel::Sender<ScheduledEvent<BUFSIZE, NCHAN>>,
    new_instances_q_rec: crossbeam::channel::Receiver<ScheduledEvent<BUFSIZE, NCHAN>>,
    block_duration: f64,
    sec_per_sample: f64,
    now: f64,
    master_reverb: MultichannelFreeverb<BUFSIZE, NCHAN>,
    master_delay: MultichannelDelay<BUFSIZE, NCHAN>,
}

impl<const BUFSIZE: usize, const NCHAN: usize> Ruffbox<BUFSIZE, NCHAN> {
    pub fn new(live_buffer: bool, life_buffer_time: f32) -> Ruffbox<BUFSIZE, NCHAN> {
        let (tx, rx): (
            Sender<ScheduledEvent<BUFSIZE, NCHAN>>,
            Receiver<ScheduledEvent<BUFSIZE, NCHAN>>,
        ) = crossbeam::channel::bounded(1000);

        // tweak some reverb values ...
        let mut rev = MultichannelFreeverb::new();
        rev.set_roomsize(0.65);
        rev.set_damp(0.43);
        rev.set_wet(1.0);

        let mut buffers = Vec::with_capacity(2000);
        let mut buffer_lengths = Vec::with_capacity(2000);

        if live_buffer {
            // create live buffer
            buffers.push(vec![0.0; (44100.0 * life_buffer_time) as usize + 3]);
            buffer_lengths.push((44100.0 * life_buffer_time) as usize);
            println!("live buf time samples: {}", buffer_lengths[0]);
            for _ in 0..10 {
                // create freeze buffers
                buffers.push(vec![0.0; (44100.0 * life_buffer_time) as usize + 3]);
                buffer_lengths.push((44100.0 * life_buffer_time) as usize);
            }
        }

        // pre-calculate a fade curve for live buffer stitching
        let bufsize = BUFSIZE;
        let stitch_size = bufsize / 4;
        let mut stitch_buffer = Vec::new();
        let mut fade_curve = Vec::new();

        let pi_inc = std::f32::consts::PI / stitch_size as f32;
        let mut pi_idx: f32 = 0.0;

        for _ in 0..stitch_size {
            stitch_buffer.push(0.0);
            fade_curve.push((-pi_idx.cos() + 1.0) / 2.0);
            pi_idx += pi_inc;
        }

        Ruffbox {
            running_instances: Vec::with_capacity(600),
            pending_events: Vec::with_capacity(600),
            buffers: buffers,
            buffer_lengths: buffer_lengths,
            live_buffer_idx: 1,
            live_buffer_current_block: 0,
            live_buffer_stitch_size: stitch_size,
            stitch_buffer: stitch_buffer,
            fade_curve: fade_curve,
            non_stitch_size: bufsize - stitch_size,
            fade_stitch_idx: 0,
            bufsize: bufsize,
            prepared_instance_map: HashMap::with_capacity(1200),
            instance_counter: AtomicCell::new(0),
            new_instances_q_send: tx,
            new_instances_q_rec: rx,
            // timing stuff
            block_duration: BUFSIZE as f64 / 44100.0,
            sec_per_sample: 1.0 / 44100.0,
            now: 0.0,
            master_reverb: rev,
            master_delay: MultichannelDelay::new(),
        }
    }

    pub fn write_samples_to_live_buffer(&mut self, samples: &[f32]) {
        for s in samples.iter() {
            self.buffers[0][self.live_buffer_idx] = *s;
            self.live_buffer_idx += 1;
            if self.live_buffer_idx >= self.buffer_lengths[0] {
                self.live_buffer_idx = 1;
            }
        }
    }

    // there HAS to be a more elegant solution for this ...
    pub fn write_sample_to_live_buffer(&mut self, sample: f32) {
        // first, overwrite old stitch region if we're at the beginning of a new block
        if self.live_buffer_current_block == 0 {
            let mut count_back_idx = self.live_buffer_idx - 1;
            for s in (0..128).rev() {
                if count_back_idx < 1 {
                    count_back_idx = self.buffer_lengths[0]; // live buffer length
                }
                self.buffers[0][count_back_idx] = self.stitch_buffer[s];
                count_back_idx -= 1;
            }
        }

        if self.live_buffer_current_block < self.non_stitch_size {
            self.buffers[0][self.live_buffer_idx] = sample;
        } else if self.live_buffer_current_block < self.bufsize {
            self.stitch_buffer[self.fade_stitch_idx] = sample;

            // stitch by fading ...
            self.buffers[0][self.live_buffer_idx] = self.buffers[0][self.live_buffer_idx]
                * self.fade_curve[self.fade_stitch_idx]
                + sample * (1.0 - self.fade_curve[self.fade_stitch_idx]);
            self.fade_stitch_idx += 1;
        }

        self.live_buffer_idx += 1;
        self.live_buffer_current_block += 1;

        if self.live_buffer_idx >= self.buffer_lengths[0] {
            self.live_buffer_idx = 1;
        }

        if self.live_buffer_current_block >= self.bufsize {
            self.live_buffer_current_block = 0;
        }

        if self.fade_stitch_idx >= self.live_buffer_stitch_size {
            self.fade_stitch_idx = 0;
        }
    }

    /// transfer contents of live buffer to freeze buffer
    pub fn freeze_buffer(&mut self, freezbuf: usize) {
        for i in 1..self.buffer_lengths[0] + 1 {
            self.buffers[freezbuf][i] = self.buffers[0][i];
        }
    }

    pub fn process(
        &mut self,
        stream_time: f64,
        track_time_internally: bool,
    ) -> [[f32; BUFSIZE]; NCHAN] {
        let mut out_buf: [[f32; BUFSIZE]; NCHAN] = [[0.0; BUFSIZE]; NCHAN];

        let mut master_delay_in: [[f32; BUFSIZE]; NCHAN] = [[0.0; BUFSIZE]; NCHAN];
        let mut master_reverb_in: [[f32; BUFSIZE]; NCHAN] = [[0.0; BUFSIZE]; NCHAN];

        if !track_time_internally {
            self.now = stream_time;
        }

        // remove finished instances ...
        self.running_instances
            .retain(|instance| !&instance.is_finished());

        // add new instances
        for new_event in self.new_instances_q_rec.try_iter() {
            if new_event.timestamp == 0.0 || new_event.timestamp == self.now {
                self.running_instances.push(new_event.source);
            //println!("now");
            } else if new_event.timestamp < self.now {
                // late events
                self.running_instances.push(new_event.source);
                // how to send out a late message ??
                // some lock-free message queue to a printer thread or something ....
                println!("late");
            } else {
                self.pending_events.push(new_event);
            }
        }

        // handle already running instances
        for running_inst in self.running_instances.iter_mut() {
            let block = running_inst.get_next_block(0, &self.buffers);

            // this should benefit from unrolling outer loop with macro ...
            for c in 0..NCHAN {
                for s in 0..BUFSIZE {
                    out_buf[c][s] += block[c][s];
                    master_reverb_in[c][s] += block[c][s] * running_inst.reverb_level();
                    master_delay_in[c][s] += block[c][s] * running_inst.delay_level();
                }
            }
        }

        // sort new events by timestamp, order of already sorted elements doesn't matter
        self.pending_events.sort_unstable_by(|a, b| b.cmp(a));
        let block_end = self.now + self.block_duration;

        // fetch event if it belongs to this block, if any ...
        while !self.pending_events.is_empty()
            && self.pending_events.last().unwrap().timestamp < block_end
        {
            let mut current_event = self.pending_events.pop().unwrap();
            //println!("on time ts: {} st: {}", current_event.timestamp, self.now);
            // calculate precise timing
            let sample_offset = (current_event.timestamp - self.now) / self.sec_per_sample;

            let block = current_event
                .source
                .get_next_block(sample_offset.round() as usize, &self.buffers);

            for c in 0..NCHAN {
                for s in 0..BUFSIZE {
                    out_buf[c][s] += block[c][s];
                    master_reverb_in[c][s] += block[c][s] * current_event.source.reverb_level();
                    master_delay_in[c][s] += block[c][s] * current_event.source.delay_level();
                }
            }

            // if length of sample event is longer than the rest of the block,
            // add to running instances
            if !current_event.source.is_finished() {
                self.running_instances.push(current_event.source);
            }
        }

        let reverb_out = self.master_reverb.process(master_reverb_in);
        let delay_out = self.master_delay.process(master_delay_in);

        //println!("{} {}", self.running_instances.len(), self.pending_events.len());

        for c in 0..NCHAN {
            for s in 0..BUFSIZE {
                out_buf[c][s] += reverb_out[c][s] + delay_out[c][s];
            }
        }

        if track_time_internally {
            self.now += self.block_duration;
        }
        //println!("now {}", self.now);

        out_buf
    }

    /// prepare a sound source instance, return instance id
    pub fn prepare_instance(
        &mut self,
        src_type: SourceType,
        timestamp: f64,
        sample_buf: usize,
    ) -> usize {
        let instance_id = self.instance_counter.fetch_add(1);

        let scheduled_event = match src_type {
            SourceType::SineOsc => {
                ScheduledEvent::new(timestamp, Box::new(SineSynth::new(44100.0)))
            }
            SourceType::SineSynth => {
                ScheduledEvent::new(timestamp, Box::new(SineSynth::new(44100.0)))
            }
	    SourceType::LFTriangleSynth => {
                ScheduledEvent::new(timestamp, Box::new(LFTriSynth::new(44100.0)))
            }
            SourceType::Sampler => ScheduledEvent::new(
                timestamp,
                Box::new(NChannelSampler::with_bufnum_len(
                    sample_buf,
                    self.buffer_lengths[sample_buf],
                    44100.0,
                )),
            ),
            SourceType::LiveSampler => ScheduledEvent::new(
                timestamp,
                Box::new(NChannelSampler::with_bufnum_len(
                    0,
                    self.buffer_lengths[0],
                    44100.0,
                )),
            ),
            SourceType::LFSawSynth => {
                ScheduledEvent::new(timestamp, Box::new(LFSawSynth::new(44100.0)))
            }
            SourceType::LFSquareSynth => {
                ScheduledEvent::new(timestamp, Box::new(LFSquareSynth::new(44100.0)))
            }
            SourceType::LFCubSynth => {
                ScheduledEvent::new(timestamp, Box::new(LFCubSynth::new(44100.0)))
            }
        };

        self.prepared_instance_map
            .insert(instance_id, scheduled_event);

        instance_id
    }

    pub fn set_instance_parameter(&mut self, instance_id: usize, par: SynthParameter, val: f32) {
        self.prepared_instance_map
            .get_mut(&instance_id)
            .unwrap()
            .set_parameter(par, val);
    }

    pub fn set_master_parameter(&mut self, par: SynthParameter, val: f32) {
        self.master_reverb.set_parameter(par, val);
        self.master_delay.set_parameter(par, val);
    }

    /// triggers a synth for buffer reference or a synth
    pub fn trigger(&mut self, instance_id: usize) {
        // add check if it actually exists !
        let scheduled_event = self.prepared_instance_map.remove(&instance_id).unwrap();
        self.new_instances_q_send.send(scheduled_event).unwrap();
    }

    /// loads a mono sample and returns the assigned buffer number
    pub fn load_sample(&mut self, samples: &[f32]) -> usize {
        self.buffer_lengths.push(samples.len() - 3); // account for interpolation samples
        self.buffers.push(samples.to_vec());
        self.buffers.len() - 1
    }

    pub fn get_now(&self) -> f64 {
        self.now
    }
}

// TEST TEST TEST
#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_stitch_stuff() {
        let mut ruff = Ruffbox::<512, 2>::new(true);
        assert_approx_eq::assert_approx_eq!(ruff.fade_curve[0], 0.0, 0.00001);
        assert_approx_eq::assert_approx_eq!(ruff.fade_curve[127], 1.0, 0.0002);

        for _ in 0..512 {
            ruff.write_sample_to_live_buffer(1.0);
        }

        for s in 0..128 {
            assert_approx_eq::assert_approx_eq!(ruff.stitch_buffer[s], 1.0, 0.0002);
        }
        assert_approx_eq::assert_approx_eq!(ruff.buffers[0][1], 1.0, 0.0002);
        assert_approx_eq::assert_approx_eq!(ruff.buffers[0][513], 0.0, 0.0002);
        assert_approx_eq::assert_approx_eq!(ruff.buffers[0][385], 1.0, 0.0002);

        for _ in 0..512 {
            ruff.write_sample_to_live_buffer(1.0);
        }

        assert_approx_eq::assert_approx_eq!(ruff.buffers[0][513], 1.0, 0.0002);
        assert_approx_eq::assert_approx_eq!(ruff.buffers[0][385], 1.0, 0.0002);
        assert_approx_eq::assert_approx_eq!(ruff.buffers[0][1024], 0.0, 0.0002);
        assert_approx_eq::assert_approx_eq!(ruff.buffers[0][896], 1.0, 0.0002);

        // write some seconds
        for _ in 0..2000 {
            for _ in 0..512 {
                ruff.write_sample_to_live_buffer(1.0);
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
        let mut ruff = Ruffbox::<512, 2>::new(false);

        let mut sample = vec![1.0; 503];
        sample[0] = 0.0;
        sample[501] = 0.0;
        sample[502] = 0.0;

        ruff.load_sample(&sample);

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
        let mut ruff = Ruffbox::<128, 2>::new(true);

        let inst = ruff.prepare_instance(SourceType::SineSynth, 0.0, 0);
        ruff.set_instance_parameter(inst, SynthParameter::PitchFrequency, 440.0);
        ruff.set_instance_parameter(inst, SynthParameter::ChannelPosition, 0.0);
        ruff.set_instance_parameter(inst, SynthParameter::Level, 1.0);
        ruff.set_instance_parameter(inst, SynthParameter::Attack, 0.0);
        ruff.set_instance_parameter(inst, SynthParameter::Sustain, 1.0);
        ruff.set_instance_parameter(inst, SynthParameter::Release, 0.0);

        ruff.trigger(inst);

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
        let mut ruff = Ruffbox::<128, 2>::new(true);

        // first point and last two points are for eventual interpolation
        let sample1 = [0.0, 0.0, 0.1, 0.2, 0.3, 0.4, 0.3, 0.2, 0.1, 0.0, 0.0, 0.0];
        let sample2 = [
            0.0, 0.0, 0.01, 0.02, 0.03, 0.04, 0.03, 0.02, 0.01, 0.0, 0.0, 0.0,
        ];

        let bnum1 = ruff.load_sample(&sample1);
        let bnum2 = ruff.load_sample(&sample2);

        ruff.process(0.0, true);

        let inst_1 = ruff.prepare_instance(SourceType::Sampler, 0.0, bnum1);
        let inst_2 = ruff.prepare_instance(SourceType::Sampler, 0.0, bnum2);

        // pan to left, neutralize
        ruff.set_instance_parameter(inst_1, SynthParameter::ChannelPosition, 0.0);
        ruff.set_instance_parameter(inst_1, SynthParameter::LowpassCutoffFrequency, 22050.0);
        ruff.set_instance_parameter(inst_1, SynthParameter::LowpassFilterDistortion, 0.0);
        ruff.set_instance_parameter(inst_1, SynthParameter::LowpassQFactor, 0.0);
        ruff.set_instance_parameter(inst_1, SynthParameter::Attack, 0.0);
        ruff.set_instance_parameter(inst_1, SynthParameter::Release, 0.0);
        ruff.set_instance_parameter(inst_1, SynthParameter::Sustain, 9.0 / 44100.0);

        ruff.set_instance_parameter(inst_2, SynthParameter::ChannelPosition, 0.0);
        ruff.set_instance_parameter(inst_2, SynthParameter::LowpassCutoffFrequency, 22050.0);
        ruff.set_instance_parameter(inst_2, SynthParameter::LowpassFilterDistortion, 0.0);
        ruff.set_instance_parameter(inst_2, SynthParameter::LowpassQFactor, 0.0);
        ruff.set_instance_parameter(inst_2, SynthParameter::Attack, 0.0);
        ruff.set_instance_parameter(inst_2, SynthParameter::Release, 0.0);
        ruff.set_instance_parameter(inst_2, SynthParameter::Sustain, 9.0 / 44100.0);

        ruff.trigger(inst_1);
        ruff.trigger(inst_2);

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
        let mut ruff = Ruffbox::<128, 2>::new(true, 3.0);

        // first point and last two points are for eventual interpolation
        let sample1 = [0.0, 0.0, 0.1, 0.2, 0.3, 0.4, 0.3, 0.2, 0.1, 0.0, 0.0, 0.0];

        let bnum1 = ruff.load_sample(&sample1);

        ruff.process(0.0, true);

        let inst_1 = ruff.prepare_instance(SourceType::Sampler, 0.0, bnum1);

        // pan to left
        ruff.set_instance_parameter(inst_1, SynthParameter::ChannelPosition, 0.0);
        ruff.set_instance_parameter(inst_1, SynthParameter::ReverbMix, 1.0);
        ruff.set_instance_parameter(inst_1, SynthParameter::LowpassCutoffFrequency, 22050.0);
        ruff.set_instance_parameter(inst_1, SynthParameter::LowpassFilterDistortion, 0.0);
        ruff.set_instance_parameter(inst_1, SynthParameter::LowpassQFactor, 0.0);
        ruff.set_instance_parameter(inst_1, SynthParameter::Attack, 0.0);
        ruff.set_instance_parameter(inst_1, SynthParameter::Release, 0.0);
        ruff.set_instance_parameter(inst_1, SynthParameter::Sustain, 9.0 / 44100.0);

        ruff.trigger(inst_1);

        let out_buf = ruff.process(0.0, true);

        for i in 0..9 {
            println!("{} {} ", out_buf[0][i], sample1[i + 1]);
            assert_approx_eq::assert_approx_eq!(out_buf[0][i], sample1[i + 1], 0.03);
        }
    }

    #[test]
    fn test_scheduled_playback() {
        let mut ruff = Ruffbox::<128, 2>::new(true);

        // block duration in seconds
        let block_duration = 0.00290249433;

        // first point and last two points are for eventual interpolation
        let sample1 = [0.0, 0.0, 0.1, 0.2, 0.3, 0.4, 0.3, 0.2, 0.1, 0.0, 0.0, 0.0];
        let sample2 = [
            0.0, 0.0, 0.01, 0.02, 0.03, 0.04, 0.03, 0.02, 0.01, 0.0, 0.0, 0.0,
        ];

        let bnum1 = ruff.load_sample(&sample1);
        let bnum2 = ruff.load_sample(&sample2);

        let inst_1 = ruff.prepare_instance(SourceType::Sampler, 0.291, bnum1);
        let inst_2 = ruff.prepare_instance(SourceType::Sampler, 0.291, bnum2);

        // pan to left
        ruff.set_instance_parameter(inst_1, SynthParameter::ChannelPosition, 0.0);
        ruff.set_instance_parameter(inst_1, SynthParameter::LowpassCutoffFrequency, 22050.0);
        ruff.set_instance_parameter(inst_1, SynthParameter::LowpassFilterDistortion, 0.0);
        ruff.set_instance_parameter(inst_1, SynthParameter::LowpassQFactor, 0.0);
        ruff.set_instance_parameter(inst_1, SynthParameter::Attack, 0.0);
        ruff.set_instance_parameter(inst_1, SynthParameter::Release, 0.0);
        ruff.set_instance_parameter(inst_1, SynthParameter::Sustain, 9.0 / 44100.0);

        ruff.set_instance_parameter(inst_2, SynthParameter::ChannelPosition, 0.0);
        ruff.set_instance_parameter(inst_2, SynthParameter::LowpassCutoffFrequency, 22050.0);
        ruff.set_instance_parameter(inst_2, SynthParameter::LowpassFilterDistortion, 0.0);
        ruff.set_instance_parameter(inst_2, SynthParameter::LowpassQFactor, 0.0);
        ruff.set_instance_parameter(inst_2, SynthParameter::Attack, 0.0);
        ruff.set_instance_parameter(inst_2, SynthParameter::Release, 0.0);
        ruff.set_instance_parameter(inst_2, SynthParameter::Sustain, 9.0 / 44100.0);

        ruff.trigger(inst_1);
        ruff.trigger(inst_2);

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
        let mut ruff = Ruffbox::<128, 2>::new(true, 3.0);

        // block duration in seconds
        let block_duration = 0.00290249433;
        let sec_per_sample = 0.00002267573;

        let sample1 = [0.0, 0.0, 0.1, 0.2, 0.3, 0.4, 0.3, 0.2, 0.1, 0.0, 0.0, 0.0];
        let sample2 = [
            0.0, 0.0, 0.01, 0.02, 0.03, 0.04, 0.03, 0.02, 0.01, 0.0, 0.0, 0.0,
        ];

        let bnum1 = ruff.load_sample(&sample1);
        let bnum2 = ruff.load_sample(&sample2);

        let inst_1 = ruff.prepare_instance(SourceType::Sampler, 0.291, bnum1);
        let inst_2 =
            ruff.prepare_instance(SourceType::Sampler, 0.291 + (4.0 * sec_per_sample), bnum2);

        // pan to left
        ruff.set_instance_parameter(inst_1, SynthParameter::ChannelPosition, 0.0);
        ruff.set_instance_parameter(inst_1, SynthParameter::LowpassCutoffFrequency, 22050.0);
        ruff.set_instance_parameter(inst_1, SynthParameter::LowpassFilterDistortion, 0.0);
        ruff.set_instance_parameter(inst_1, SynthParameter::LowpassQFactor, 0.0);
        ruff.set_instance_parameter(inst_1, SynthParameter::Attack, 0.0);
        ruff.set_instance_parameter(inst_1, SynthParameter::Release, 0.0);
        ruff.set_instance_parameter(inst_1, SynthParameter::Sustain, 9.0 / 44100.0);

        ruff.set_instance_parameter(inst_2, SynthParameter::ChannelPosition, 0.0);
        ruff.set_instance_parameter(inst_2, SynthParameter::LowpassCutoffFrequency, 22050.0);
        ruff.set_instance_parameter(inst_2, SynthParameter::LowpassFilterDistortion, 0.0);
        ruff.set_instance_parameter(inst_2, SynthParameter::LowpassQFactor, 0.0);
        ruff.set_instance_parameter(inst_2, SynthParameter::Attack, 0.0);
        ruff.set_instance_parameter(inst_2, SynthParameter::Release, 0.0);
        ruff.set_instance_parameter(inst_2, SynthParameter::Sustain, 9.0 / 44100.0);

        ruff.trigger(inst_1);
        ruff.trigger(inst_2);

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
        let mut ruff = Ruffbox::<128, 2>::new(true, 3.0);

        // block duration in seconds
        let block_duration = 0.00290249433;

        let sample1 = [0.0, 0.0, 0.1, 0.2, 0.3, 0.4, 0.3, 0.2, 0.1, 0.0, 0.0, 0.0];
        let sample2 = [
            0.0, 0.0, 0.01, 0.02, 0.03, 0.04, 0.03, 0.02, 0.01, 0.0, 0.0, 0.0,
        ];

        let bnum1 = ruff.load_sample(&sample1);
        let bnum2 = ruff.load_sample(&sample2);

        // schedule two samples ahead, so they should  occur in different blocks
        // first sample should appear in block 100

        // second sample should appear ten blocks later
        let second_sample_timestamp = 0.291 + (10.0 * block_duration);

        let inst_1 = ruff.prepare_instance(SourceType::Sampler, 0.291, bnum1);
        let inst_2 = ruff.prepare_instance(SourceType::Sampler, second_sample_timestamp, bnum2);

        // pan to left
        ruff.set_instance_parameter(inst_1, SynthParameter::ChannelPosition, 0.0);
        ruff.set_instance_parameter(inst_1, SynthParameter::LowpassCutoffFrequency, 22050.0);
        ruff.set_instance_parameter(inst_1, SynthParameter::LowpassFilterDistortion, 0.0);
        ruff.set_instance_parameter(inst_1, SynthParameter::LowpassQFactor, 0.0);
        ruff.set_instance_parameter(inst_1, SynthParameter::Attack, 0.0);
        ruff.set_instance_parameter(inst_1, SynthParameter::Release, 0.0);
        ruff.set_instance_parameter(inst_1, SynthParameter::Sustain, 9.0 / 44100.0);

        ruff.set_instance_parameter(inst_2, SynthParameter::ChannelPosition, 0.0);
        ruff.set_instance_parameter(inst_2, SynthParameter::LowpassCutoffFrequency, 22050.0);
        ruff.set_instance_parameter(inst_2, SynthParameter::LowpassFilterDistortion, 0.0);
        ruff.set_instance_parameter(inst_2, SynthParameter::LowpassQFactor, 0.0);
        ruff.set_instance_parameter(inst_2, SynthParameter::Attack, 0.0);
        ruff.set_instance_parameter(inst_2, SynthParameter::Release, 0.0);
        ruff.set_instance_parameter(inst_2, SynthParameter::Sustain, 9.0 / 44100.0);

        ruff.trigger(inst_1);
        ruff.trigger(inst_2);

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
        let mut ruff = Ruffbox::<128, 2>::new(true);

        let sample1 = [0.0, 0.0, 0.1, 0.2, 0.3, 0.4, 0.3, 0.2, 0.1, 0.0, 0.0, 0.0];

        let bnum1 = ruff.load_sample(&sample1);

        ruff.process(0.0, false);

        let inst_1 = ruff.prepare_instance(SourceType::Sampler, 0.1, bnum1);

        // pan to left
        ruff.set_instance_parameter(inst_1, SynthParameter::ChannelPosition, 0.0);
        ruff.set_instance_parameter(inst_1, SynthParameter::LowpassCutoffFrequency, 22050.0);
        ruff.set_instance_parameter(inst_1, SynthParameter::LowpassFilterDistortion, 0.0);
        ruff.set_instance_parameter(inst_1, SynthParameter::LowpassQFactor, 0.0);
        ruff.set_instance_parameter(inst_1, SynthParameter::Attack, 0.0);
        ruff.set_instance_parameter(inst_1, SynthParameter::Release, 0.0);
        ruff.set_instance_parameter(inst_1, SynthParameter::Sustain, 9.0 / 44100.0);

        ruff.trigger(inst_1);

        // process after the instance's trigger time
        let out_buf = ruff.process(0.101, false);

        for i in 0..9 {
            assert_approx_eq::assert_approx_eq!(out_buf[0][i], sample1[i + 1], 0.03);
        }
    }
}
