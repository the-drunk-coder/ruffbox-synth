use rubato::{FftFixedIn, Resampler};

// crossbeam for the event queue
use crossbeam::atomic::AtomicCell;

use std::sync::Arc;

use crate::building_blocks::ambisonics::binauralizer_o1::BinauralizerO1;
use crate::building_blocks::delay::MultichannelDelay;
use crate::building_blocks::reverb::convolution::MultichannelConvolutionReverb;
use crate::building_blocks::reverb::freeverb::MultichannelFreeverb;
use crate::building_blocks::{MultichannelReverb, SampleBuffer, Synth};

use crate::ruffbox::{ControlMessage, ReverbMode, ScheduledEvent};

use crate::ruffbox::ScheduledSource;

pub(crate) struct LiveBufferMetadata<const BUFSIZE: usize> {
    live_buffer_idx: usize,
    //pub(crate) stitch_buffer_incoming: Vec<f32>,
    pub(crate) stitch_buffer_previous: Vec<f32>,
    accum_buf: [f32; BUFSIZE],
    accum_buf_idx: usize,
}

/// ambisonic binaural module (order 1 for now)
pub struct AmbisonicBinaural<const BUFSIZE: usize, const NCHAN: usize> {
    running_instances: Vec<Box<dyn Synth<BUFSIZE, 4> + Send + Sync>>, // first order ambisonic sources
    // has to be n-channel unfotunately ..
    pending_events: Vec<ScheduledEvent<BUFSIZE, NCHAN>>,
    // this has to do until I manage to implement a proper ambisonic reverb ...
    binauralizer: BinauralizerO1<BUFSIZE>,
    binauralizer_rev: BinauralizerO1<BUFSIZE>,
    ambi_master: [[f32; BUFSIZE]; 4],
    ambi_reverb_in: [[f32; BUFSIZE]; 4],
}

impl<const BUFSIZE: usize, const NCHAN: usize> AmbisonicBinaural<BUFSIZE, NCHAN> {
    pub fn new(samplerate: f32) -> Self {
        AmbisonicBinaural {
            running_instances: Vec::with_capacity(600),
            pending_events: Vec::with_capacity(600),
            binauralizer: BinauralizerO1::default_filter(samplerate),
            binauralizer_rev: BinauralizerO1::default_filter(samplerate),
            ambi_master: [[0.0; BUFSIZE]; 4],
            ambi_reverb_in: [[0.0; BUFSIZE]; 4],
        }
    }
}

/// This is the "Playhead", that is, the part you use in the
/// output callback funtion of your application
pub struct RuffboxPlayhead<const BUFSIZE: usize, const NCHAN: usize> {
    running_instances: Vec<Box<dyn Synth<BUFSIZE, NCHAN> + Send + Sync>>,
    pending_events: Vec<ScheduledEvent<BUFSIZE, NCHAN>>,
    ambisonic_binaural: Option<AmbisonicBinaural<BUFSIZE, NCHAN>>,
    pub(crate) buffers: Vec<SampleBuffer>, // crate public for test
    pub(crate) buffer_lengths: Vec<usize>, // crate public for test
    max_buffers: usize,
    stitch_size: usize,
    pub(crate) fade_curve: Vec<f32>, // crate public for test
    pub(crate) live_buffer_metadata: Vec<LiveBufferMetadata<BUFSIZE>>,
    samplerate: f32,
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
        live_buffers: usize,
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
        let mut buffers = Vec::new();
        for _ in 0..max_buffers {
            // with placeholders ...
            buffers.push(SampleBuffer::Placeholder);
        }
        // init buffer lengths
        let mut buffer_lengths = vec![0; max_buffers];

        //println!("max num buffers {} {}", buffers.len(), max_buffers);

        let stitch_size = BUFSIZE / 4;
        let mut live_buffer_metadata = Vec::new();
        let mut fade_curve = Vec::new();

        if live_buffers > 0 {
            // pre-calculate a fade curve for live buffer stitching
            let pi_inc = std::f32::consts::PI / stitch_size as f32;
            let mut pi_idx: f32 = 0.0;

            for _ in 0..stitch_size {
                // FADE-IN-CURVE
                fade_curve.push((-pi_idx.cos() + 1.0) / 2.0);
                pi_idx += pi_inc;
            }

            // one stitch buffer per live buffer
            for _ in 0..live_buffers {
                live_buffer_metadata.push(LiveBufferMetadata {
                    live_buffer_idx: 2, // let room for interpolation
                    //stitch_buffer_incoming: vec![0.0; stitch_size],
                    stitch_buffer_previous: vec![0.0; stitch_size],
                    accum_buf: [0.0; BUFSIZE],
                    accum_buf_idx: 0,
                });
            }
            // create live buffers and freeze buffers
            for b in 0..live_buffers + freeze_buffers {
                // two interpolation samples in each direction ...
                buffers[b] =
                    SampleBuffer::Mono(vec![0.0; (samplerate * live_buffer_time) as usize + 4]);
                buffer_lengths[b] = (samplerate * live_buffer_time) as usize;
            }

            println!("live buf time samples: {}", buffer_lengths[0]);
        }

        RuffboxPlayhead {
            running_instances: Vec::with_capacity(600),
            pending_events: Vec::with_capacity(600),
            ambisonic_binaural: None,
            buffers,
            buffer_lengths,
            max_buffers,
            live_buffer_metadata,
            fade_curve,
            stitch_size,
            samplerate: samplerate as f32,
            control_q_rec: rx,
            // timing stuff
            block_duration: BUFSIZE as f64 / samplerate,
            sec_per_sample: 1.0 / samplerate,
            now: Arc::clone(now),
            master_reverb: rev,
            master_delay: MultichannelDelay::new(samplerate as f32),
        }
    }

    pub fn enable_ambisonics_binaural(&mut self) {
        println!("activate ambisonic-binaural module");
        self.ambisonic_binaural = Some(AmbisonicBinaural::new(self.samplerate));
    }

    pub fn write_samples_to_live_buffer(&mut self, bufnum: usize) {
        // so far we only allow writing to a mono buffer, one input at a time
        if let Some(SampleBuffer::Mono(buf)) = self.buffers.get_mut(bufnum) {
            // WITHOUT interpolation samples
            let buflen = self.buffer_lengths[bufnum];
            let bufidx = self.live_buffer_metadata[bufnum].live_buffer_idx;

            // make sure bufidx is always bigger than 1 and smaller than
            // buflen + 2

            // calculate start point, keeping interpolation samples in mind
            let tmp_idx = bufidx - 2; // index is >= 2, always (interpolation)
            let mut cur_idx = if tmp_idx >= self.stitch_size {
                bufidx - self.stitch_size
            } else {
                let tmp = self.stitch_size - tmp_idx;
                (buflen - tmp) + 2 // add interp. samples
            };

            /*
            assert!(cur_idx >= 2);
            assert!(
                cur_idx - 2 < buflen,
                "cur {cur_idx} len {buflen} tmp {tmp_idx}"
            );*/

            for i in 0..self.stitch_size {
                buf[cur_idx] = self.live_buffer_metadata[bufnum].stitch_buffer_previous[i];
                cur_idx += 1;
                // flip if necessary
                if cur_idx - 2 >= buflen {
                    //println!("FLIP 1");
                    cur_idx = 2;
                }
            }

            //assert!(cur_idx >= 2);
            //assert!(cur_idx - 2 < buflen);

            // back to where we were ...
            //assert!(cur_idx == bufidx, "curid {cur_idx} bufid {bufidx}");

            let buf_head = BUFSIZE - self.stitch_size;

            for i in 0..buf_head {
                buf[cur_idx] = self.live_buffer_metadata[bufnum].accum_buf[i];
                cur_idx += 1;
                // flip if necessary
                if cur_idx - 2 >= buflen {
                    //println!("FLIP 2");
                    cur_idx = 2;
                }
            }

            //assert!(cur_idx >= 2);
            //assert!(cur_idx - 2 < buflen);

            // keep for later
            for i in buf_head..BUFSIZE {
                self.live_buffer_metadata[bufnum].stitch_buffer_previous[i - buf_head] =
                    self.live_buffer_metadata[bufnum].accum_buf[i]
            }

            for i in 0..self.stitch_size {
                let gain = self.fade_curve[i];
                buf[cur_idx] = buf[cur_idx] * gain
                    + self.live_buffer_metadata[bufnum].accum_buf[buf_head + i] * (1.0 - gain);
                cur_idx += 1;
                // flip if necessary
                if cur_idx - 2 >= buflen {
                    //println!("FLIP 3");
                    cur_idx = 2;
                }
            }

            //assert!(cur_idx >= 2);
            //assert!(cur_idx - 2 < buflen);

            self.live_buffer_metadata[bufnum].live_buffer_idx = cur_idx;
        }
    }

    pub fn write_sample_to_live_buffer(&mut self, bufnum: usize, sample: f32) {
        let mut idx = self.live_buffer_metadata[bufnum].accum_buf_idx;
        self.live_buffer_metadata[bufnum].accum_buf[idx] = sample;
        idx += 1;
        if idx == BUFSIZE {
            self.write_samples_to_live_buffer(bufnum);
            self.live_buffer_metadata[bufnum].accum_buf_idx = 0;
        } else {
            self.live_buffer_metadata[bufnum].accum_buf_idx = idx;
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

        // clear ambi master if necessary
        if let Some(ambi_module) = self.ambisonic_binaural.as_mut() {
            ambi_module.ambi_master = [[0.0; BUFSIZE]; 4];
            ambi_module.ambi_reverb_in = [[0.0; BUFSIZE]; 4];
        }

        let now = if !track_time_internally {
            self.now.store(stream_time);
            stream_time
        } else {
            self.now.load()
        };

        // remove finished instances ...
        self.running_instances
            .retain(|instance| !&instance.is_finished());

        // in case we have ambisonic mode enabled
        if let Some(ambi_module) = self.ambisonic_binaural.as_mut() {
            ambi_module
                .running_instances
                .retain(|instance| !&instance.is_finished());
        }

        for cm in self.control_q_rec.try_iter() {
            match cm {
                ControlMessage::SetGlobalParamOrModulator(par, val) => {
                    // BAD CLONE in audio thread, but it should happen only very rarely ...
                    self.master_reverb.set_param_or_modulator(par, val.clone());
                    self.master_delay.set_param_or_modulator(par, val);
                }
                ControlMessage::ScheduleEvent(sched_event) => {
                    // add new instances
                    match sched_event.source {
                        ScheduledSource::Channel(src) => {
                            if sched_event.timestamp == 0.0 || sched_event.timestamp == now {
                                self.running_instances.push(src);
                            //println!("now");
                            } else if sched_event.timestamp < now {
                                // late events
                                self.running_instances.push(src);
                                // how to send out a late message ??
                                // some lock-free message queue to a printer thread or something ....
                                println!("late");
                            } else {
                                self.pending_events.push(ScheduledEvent {
                                    timestamp: sched_event.timestamp,
                                    source: ScheduledSource::Channel(src),
                                });
                            }
                        }
                        // handle ambisonic sources ...
                        ScheduledSource::Ambi(src) => {
                            if let Some(ambi_module) = self.ambisonic_binaural.as_mut() {
                                if sched_event.timestamp == 0.0 || sched_event.timestamp == now {
                                    ambi_module.running_instances.push(src);
                                //println!("now");
                                } else if sched_event.timestamp < now {
                                    // late events
                                    ambi_module.running_instances.push(src);
                                    // how to send out a late message ??
                                    // some lock-free message queue to a printer thread or something ....
                                    println!("ambi late");
                                } else {
                                    ambi_module.pending_events.push(ScheduledEvent {
                                        timestamp: sched_event.timestamp,
                                        source: ScheduledSource::Ambi(src),
                                    });
                                }
                            }
                        }
                    }
                }
                ControlMessage::LoadSample(id, len, content) => {
                    if id < self.max_buffers {
                        self.buffers[id] = content; // transfer to samples
                        self.buffer_lengths[id] = len;
                    }
                }
                ControlMessage::FreezeBuffer(fb, ib) => {
                    // start at one to account for interpolation sample.
                    if let Ok([SampleBuffer::Mono(inbuf), SampleBuffer::Mono(freezbuf)]) =
                        self.buffers.get_disjoint_mut([ib, fb])
                    {
                        freezbuf[1..(self.buffer_lengths[ib] + 1)]
                            .copy_from_slice(&inbuf[1..(self.buffer_lengths[ib] + 1)]);
                    }
                }
                ControlMessage::FreezeAddBuffer(fb, ib) => {
                    // start at one to account for interpolation sample.
                    if let Ok([SampleBuffer::Mono(inbuf), SampleBuffer::Mono(freezbuf)]) =
                        self.buffers.get_disjoint_mut([ib, fb])
                    {
                        for i in 1..(self.buffer_lengths[ib] + 1) {
                            freezbuf[i] += inbuf[i];
                        }
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

        if let Some(ambi_module) = self.ambisonic_binaural.as_mut() {
            for running_inst in ambi_module.running_instances.iter_mut() {
                let ambi_block = running_inst.get_next_block(0, &self.buffers);

                // this should benefit from unrolling outer loop with macro ...
                for c in 0..4 {
                    for s in 0..BUFSIZE {
                        ambi_module.ambi_master[c][s] += ambi_block[c][s];
                        ambi_module.ambi_reverb_in[c][s] +=
                            ambi_block[c][s] * running_inst.reverb_level();
                    }
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
            let current_event = self.pending_events.pop().unwrap();
            //println!("on time ts: {} st: {}", current_event.timestamp, self.now);
            // calculate precise timing
            let sample_offset = (current_event.timestamp - now) / self.sec_per_sample;
            if let ScheduledSource::Channel(mut src) = current_event.source {
                let block = src.get_next_block(sample_offset.round() as usize, &self.buffers);

                for c in 0..NCHAN {
                    for s in 0..BUFSIZE {
                        out_buf[c][s] += block[c][s];
                        master_reverb_in[c][s] += block[c][s] * src.reverb_level();
                        master_delay_in[c][s] += block[c][s] * src.delay_level();
                    }
                }

                // if length of sample event is longer than the rest of the block,
                // add to running instances
                if !src.is_finished() {
                    self.running_instances.push(src);
                }
            }
        }

        if let Some(ambi_module) = self.ambisonic_binaural.as_mut() {
            // sort new events by timestamp, order of already sorted elements doesn't matter
            ambi_module.pending_events.sort_unstable_by(|a, b| b.cmp(a));

            // fetch event if it belongs to this block, if any ...
            while !ambi_module.pending_events.is_empty()
                && ambi_module.pending_events.last().unwrap().timestamp < block_end
            {
                let current_event = ambi_module.pending_events.pop().unwrap();
                //println!("on time ts: {} st: {}", current_event.timestamp, self.now);
                // calculate precise timing
                let sample_offset = (current_event.timestamp - now) / self.sec_per_sample;
                if let ScheduledSource::Ambi(mut src) = current_event.source {
                    let ambi_block =
                        src.get_next_block(sample_offset.round() as usize, &self.buffers);

                    for c in 0..4 {
                        for s in 0..BUFSIZE {
                            ambi_module.ambi_master[c][s] += ambi_block[c][s];
                            ambi_module.ambi_reverb_in[c][s] +=
                                ambi_block[c][s] * src.reverb_level();
                        }
                    }

                    // if length of sample event is longer than the rest of the block,
                    // add to running instances
                    if !src.is_finished() {
                        ambi_module.running_instances.push(src);
                    }
                }
            }

            let block = ambi_module
                .binauralizer
                .binauralize(ambi_module.ambi_master);

            // this has to do for a reverb until I manage to implement a proper ambisonic reverb ...
            let block_rev = ambi_module
                .binauralizer_rev
                .binauralize(ambi_module.ambi_reverb_in);

            // mix binauralized block in with master
            for c in 0..NCHAN {
                for s in 0..BUFSIZE {
                    out_buf[c][s] += block[c][s];
                    master_reverb_in[c][s] += block_rev[c][s];
                }
            }
        }

        let reverb_out = self.master_reverb.process(master_reverb_in);
        let delay_out = self.master_delay.process(master_delay_in, &self.buffers);

        for c in 0..NCHAN {
            for s in 0..BUFSIZE {
                out_buf[c][s] += reverb_out[c][s] + delay_out[c][s];
            }
        }

        if track_time_internally {
            self.now.store(now + self.block_duration);
        }

        out_buf
    }
}
