use rubato::{FftFixedIn, Resampler};

// crossbeam for the event queue
use crossbeam::atomic::AtomicCell;

use std::sync::Arc;

use crate::ruffbox::synth::convolution_reverb::MultichannelConvolutionReverb;
use crate::ruffbox::synth::delay::MultichannelDelay;
use crate::ruffbox::synth::freeverb::MultichannelFreeverb;
use crate::ruffbox::synth::MultichannelReverb;
use crate::ruffbox::synth::Synth;
use crate::ruffbox::ControlMessage;
use crate::ruffbox::ReverbMode;
use crate::ruffbox::ScheduledEvent;

/// This is the "Playhead", that is, the part you use in the
/// output callback funtion of your application
pub struct RuffboxPlayhead<const BUFSIZE: usize, const NCHAN: usize> {
    running_instances: Vec<Box<dyn Synth<BUFSIZE, NCHAN> + Send + Sync>>,
    pending_events: Vec<ScheduledEvent<BUFSIZE, NCHAN>>,
    pub(crate) buffers: Vec<Vec<f32>>,     // crate public for test
    pub(crate) buffer_lengths: Vec<usize>, // crate public for test
    max_buffers: usize,
    live_buffer_idx: usize,
    live_buffer_current_block: usize,
    live_buffer_stitch_size: usize,
    non_stitch_size: usize,
    fade_stitch_idx: usize,
    pub(crate) fade_curve: Vec<f32>, // crate public for test
    pub(crate) stitch_buffer: Vec<f32>,
    bufsize: usize,
    control_q_rec: crossbeam::channel::Receiver<ControlMessage<BUFSIZE, NCHAN>>,
    block_duration: f64,
    sec_per_sample: f64,
    now: Arc<AtomicCell<f64>>,
    master_reverb: Box<dyn MultichannelReverb<BUFSIZE, NCHAN> + Send + Sync>,
    master_delay: MultichannelDelay<BUFSIZE, NCHAN>,
}

impl<const BUFSIZE: usize, const NCHAN: usize> RuffboxPlayhead<BUFSIZE, NCHAN> {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        live_buffer: bool,
        live_buffer_time: f64,
        reverb_mode: &ReverbMode,
        samplerate: f64,
        max_buffers: usize,
        freeze_buffers: usize,
        now: &Arc<AtomicCell<f64>>,
        rx: crossbeam::channel::Receiver<ControlMessage<BUFSIZE, NCHAN>>,
    ) -> RuffboxPlayhead<BUFSIZE, NCHAN> {
        // create reverb
        let rev: Box<dyn MultichannelReverb<BUFSIZE, NCHAN> + Send + Sync> = match reverb_mode {
            ReverbMode::FreeVerb => {
                let mut mrev = MultichannelFreeverb::new(samplerate as f32);
                // tweak some reverb values for freeverb
                mrev.set_roomsize(0.65);
                mrev.set_damp(0.43);
                mrev.set_wet(1.0);
                Box::new(mrev)
            }
            ReverbMode::Convolution(ir, sr) => {
                let mut ir_clone = ir.clone();
                // resample IR if needed ...
                if *sr as f64 != samplerate {
                    // zero-pad for resampling blocks
                    if (ir.len() as f32 % 1024.0) > 0.0 {
                        let diff = 1024 - (ir.len() % 1024);
                        ir_clone.append(&mut vec![0.0; diff]);
                    }

                    let mut ir_resampled: Vec<f32> = Vec::new();
                    let mut resampler =
                        FftFixedIn::<f32>::new(*sr as usize, samplerate as usize, 1024, 1, 1);

                    let num_chunks = ir.len() / 1024;
                    for chunk in 0..num_chunks {
                        let chunk = vec![ir_clone[(1024 * chunk)..(1024 * (chunk + 1))].to_vec()];
                        let mut waves_out = resampler.process(&chunk).unwrap();
                        ir_resampled.append(&mut waves_out[0]);
                    }
                    Box::new(MultichannelConvolutionReverb::with_ir(&ir_resampled))
                } else {
                    Box::new(MultichannelConvolutionReverb::with_ir(ir))
                }
            }
        };

        // init buffer memory
        let mut buffers = vec![vec![0.0]; max_buffers];
        // init buffer lengths
        let mut buffer_lengths = vec![0; max_buffers];

        //println!("max num buffers {} {}", buffers.len(), max_buffers);

        if live_buffer {
            // create live buffer
            buffers[0] = vec![0.0; (samplerate * live_buffer_time) as usize + 3];
            buffer_lengths[0] = (samplerate * live_buffer_time) as usize;
            println!("live buf time samples: {}", buffer_lengths[0]);
            for b in 1..freeze_buffers + 1 {
                // create freeze buffers
                buffers[b] = vec![0.0; (samplerate * live_buffer_time) as usize + 3];
                buffer_lengths[b] = (samplerate * live_buffer_time) as usize;
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

        RuffboxPlayhead {
            running_instances: Vec::with_capacity(600),
            pending_events: Vec::with_capacity(600),
            buffers,
            buffer_lengths,
            max_buffers,
            live_buffer_idx: 1,
            live_buffer_current_block: 0,
            live_buffer_stitch_size: stitch_size,
            stitch_buffer,
            fade_curve,
            non_stitch_size: bufsize - stitch_size,
            fade_stitch_idx: 0,
            bufsize,
            control_q_rec: rx,
            // timing stuff
            block_duration: BUFSIZE as f64 / samplerate,
            sec_per_sample: 1.0 / samplerate,
            now: Arc::clone(now),
            master_reverb: rev,
            master_delay: MultichannelDelay::new(samplerate as f32),
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
            for s in (0..self.stitch_buffer.len()).rev() {
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

    pub fn process(
        &mut self,
        stream_time: f64,
        track_time_internally: bool,
    ) -> [[f32; BUFSIZE]; NCHAN] {
        let mut out_buf: [[f32; BUFSIZE]; NCHAN] = [[0.0; BUFSIZE]; NCHAN];

        let mut master_delay_in: [[f32; BUFSIZE]; NCHAN] = [[0.0; BUFSIZE]; NCHAN];
        let mut master_reverb_in: [[f32; BUFSIZE]; NCHAN] = [[0.0; BUFSIZE]; NCHAN];

        let now = if !track_time_internally {
            self.now.store(stream_time);
            stream_time
        } else {
            self.now.load()
        };

        // remove finished instances ...
        self.running_instances
            .retain(|instance| !&instance.is_finished());

        for cm in self.control_q_rec.try_iter() {
            match cm {
                ControlMessage::SetGlobalParam(par, val) => {
                    self.master_reverb.set_parameter(par, val);
                    self.master_delay.set_parameter(par, val);
                }
                ControlMessage::ScheduleEvent(new_event) => {
                    // add new instances
                    if new_event.timestamp == 0.0 || new_event.timestamp == now {
                        self.running_instances.push(new_event.source);
                    //println!("now");
                    } else if new_event.timestamp < now {
                        // late events
                        self.running_instances.push(new_event.source);
                        // how to send out a late message ??
                        // some lock-free message queue to a printer thread or something ....
                        println!("late");
                    } else {
                        self.pending_events.push(new_event);
                    }
                }
                ControlMessage::LoadSample(id, len, content) => {
                    if id < self.max_buffers {
                        self.buffers[id] = content; // transfer to samples
                        self.buffer_lengths[id] = len;
                    }
                }
                ControlMessage::FreezeBuffer(fb) => {
                    for i in 1..self.buffer_lengths[0] + 1 {
                        self.buffers[fb][i] = self.buffers[0][i];
                    }
                }
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
        let block_end = now + self.block_duration;

        // fetch event if it belongs to this block, if any ...
        while !self.pending_events.is_empty()
            && self.pending_events.last().unwrap().timestamp < block_end
        {
            let mut current_event = self.pending_events.pop().unwrap();
            //println!("on time ts: {} st: {}", current_event.timestamp, self.now);
            // calculate precise timing
            let sample_offset = (current_event.timestamp - now) / self.sec_per_sample;

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
            self.now.store(now + self.block_duration);
        }
        //println!("now {}", self.now);

        out_buf
    }
}
