/* --- loonixtunesv2/src/audio/io/resample.rs | resample --- */
/* Uses FFmpeg's libswresample for audio resampling - FFmpeg 6+ API */

use ringbuf::traits::Producer;
use ringbuf::HeapProd;
use std::sync::atomic::{AtomicBool, Ordering};

pub struct StereoResampler {
    swr_ctx: *mut ffmpeg_next::sys::SwrContext,
    input_rate: i32,
    output_rate: i32,
}

impl StereoResampler {
    pub fn new(input_rate: i32, output_rate: i32) -> Option<Self> {
        unsafe {
            let ctx = ffmpeg_next::sys::swr_alloc();
            if ctx.is_null() {
                return None;
            }

            let ctx_void = ctx as *mut libc::c_void;

            // Input sample format (FLTP = Float Planar)
            ffmpeg_next::sys::av_opt_set_sample_fmt(
                ctx_void,
                b"in_sample_fmt\0".as_ptr() as *const i8,
                ffmpeg_next::sys::AVSampleFormat::AV_SAMPLE_FMT_FLTP,
                0,
            );

            // Output sample format
            ffmpeg_next::sys::av_opt_set_sample_fmt(
                ctx_void,
                b"out_sample_fmt\0".as_ptr() as *const i8,
                ffmpeg_next::sys::AVSampleFormat::AV_SAMPLE_FMT_FLTP,
                0,
            );

            // Input sample rate
            ffmpeg_next::sys::av_opt_set_int(
                ctx_void,
                b"in_sample_rate\0".as_ptr() as *const i8,
                input_rate as i64,
                0,
            );

            // Output sample rate
            ffmpeg_next::sys::av_opt_set_int(
                ctx_void,
                b"out_sample_rate\0".as_ptr() as *const i8,
                output_rate as i64,
                0,
            );

            // Channel layout using NEW AVChannelLayout API
            let mut in_layout: ffmpeg_next::sys::AVChannelLayout = std::mem::zeroed();
            let mut out_layout: ffmpeg_next::sys::AVChannelLayout = std::mem::zeroed();

            ffmpeg_next::sys::av_channel_layout_default(&mut in_layout, 2);
            ffmpeg_next::sys::av_channel_layout_default(&mut out_layout, 2);

            ffmpeg_next::sys::av_opt_set_chlayout(
                ctx_void,
                b"in_chlayout\0".as_ptr() as *const i8,
                &in_layout,
                0,
            );

            ffmpeg_next::sys::av_opt_set_chlayout(
                ctx_void,
                b"out_chlayout\0".as_ptr() as *const i8,
                &out_layout,
                0,
            );

            // Initialize the resampler
            if ffmpeg_next::sys::swr_init(ctx) < 0 {
                ffmpeg_next::sys::swr_free(&mut (ctx as *mut _));
                return None;
            }

            Some(Self {
                swr_ctx: ctx,
                input_rate,
                output_rate,
            })
        }
    }

    pub fn process(&mut self, input: &[[f32; 2]]) -> Vec<[f32; 2]> {
        let in_frames = input.len();
        if in_frames == 0 {
            return Vec::new();
        }

        let max_output =
            (in_frames as f64 * self.output_rate as f64 / self.input_rate as f64) as usize + 64;

        // === Split planar input ===
        let mut in_left: Vec<f32> = Vec::with_capacity(in_frames);
        let mut in_right: Vec<f32> = Vec::with_capacity(in_frames);

        for frame in input {
            in_left.push(frame[0]);
            in_right.push(frame[1]);
        }

        let in_data: [*const u8; 2] = [
            in_left.as_ptr() as *const u8,
            in_right.as_ptr() as *const u8,
        ];

        // === Allocate planar output ===
        let mut out_left: Vec<f32> = vec![0.0; max_output];
        let mut out_right: Vec<f32> = vec![0.0; max_output];

        let mut out_data: [*mut u8; 2] = [
            out_left.as_mut_ptr() as *mut u8,
            out_right.as_mut_ptr() as *mut u8,
        ];

        let converted = unsafe {
            ffmpeg_next::sys::swr_convert(
                self.swr_ctx,
                out_data.as_mut_ptr(),
                max_output as i32,
                in_data.as_ptr(),
                in_frames as i32,
            )
        };

        if converted <= 0 {
            return Vec::new();
        }

        let converted = converted as usize;

        // === Interleave back ===
        let mut result = Vec::with_capacity(converted);
        for i in 0..converted {
            result.push([out_left[i], out_right[i]]);
        }

        result
    }

    pub fn drain(&mut self) -> Vec<[f32; 2]> {
        let max_output = 1024;

        let mut out_left: Vec<f32> = vec![0.0; max_output];
        let mut out_right: Vec<f32> = vec![0.0; max_output];

        let mut out_data: [*mut u8; 2] = [
            out_left.as_mut_ptr() as *mut u8,
            out_right.as_mut_ptr() as *mut u8,
        ];

        let drained = unsafe {
            ffmpeg_next::sys::swr_convert(
                self.swr_ctx,
                out_data.as_mut_ptr(),
                max_output as i32,
                std::ptr::null(),
                0,
            )
        };

        if drained <= 0 {
            return Vec::new();
        }

        let drained = drained as usize;

        let mut result = Vec::with_capacity(drained);
        for i in 0..drained {
            result.push([out_left[i], out_right[i]]);
        }

        result
    }
}

impl Drop for StereoResampler {
    fn drop(&mut self) {
        unsafe {
            ffmpeg_next::sys::swr_free(&mut self.swr_ctx);
        }
    }
}

pub fn create_resampler(input_rate: f64, output_rate: f64) -> Option<StereoResampler> {
    StereoResampler::new(input_rate as i32, output_rate as i32)
}

pub fn process_frame(
    raw_data: &[u8],
    resampler: &mut StereoResampler,
    producer: &mut HeapProd<f32>,
    total_decoded_samples: &mut u64,
) {
    if raw_data.is_empty() || raw_data.len() % 4 != 0 {
        return;
    }

    let sample_count = raw_data.len() / 4;
    let input_flat: &[f32] =
        unsafe { std::slice::from_raw_parts(raw_data.as_ptr() as *const f32, sample_count) };

    let input_frames = input_flat.len() / 2;
    let input_stereo: &[[f32; 2]] =
        unsafe { std::slice::from_raw_parts(input_flat.as_ptr() as *const [f32; 2], input_frames) };

    let processed = resampler.process(input_stereo);

    if !processed.is_empty() {
        push_output(&processed, processed.len(), producer, total_decoded_samples);
    }
}

pub fn process_frame_buffered(
    raw_data: &[u8],
    resampler: &mut StereoResampler,
    producer: &mut HeapProd<f32>,
    total_decoded_samples: &mut u64,
    buffered: &mut u64,
) {
    if raw_data.is_empty() || raw_data.len() % 4 != 0 {
        return;
    }

    let sample_count = raw_data.len() / 4;
    let input_flat: &[f32] =
        unsafe { std::slice::from_raw_parts(raw_data.as_ptr() as *const f32, sample_count) };

    let input_frames = input_flat.len() / 2;
    let input_stereo: &[[f32; 2]] =
        unsafe { std::slice::from_raw_parts(input_flat.as_ptr() as *const [f32; 2], input_frames) };

    let processed = resampler.process(input_stereo);

    if !processed.is_empty() {
        let flat_len = processed.len() * 2;
        *buffered += flat_len as u64;
        push_output(&processed, processed.len(), producer, total_decoded_samples);
    }
}

pub fn drain(
    resampler: &mut StereoResampler,
    producer: &mut HeapProd<f32>,
    total_decoded_samples: &mut u64,
    should_stop: &AtomicBool,
) {
    loop {
        if should_stop.load(Ordering::Relaxed) {
            break;
        }

        let output = resampler.drain();
        if output.is_empty() {
            break;
        }

        push_output(&output, output.len(), producer, total_decoded_samples);
    }
}

fn push_output(
    output_stereo: &[[f32; 2]],
    output_frames: usize,
    producer: &mut HeapProd<f32>,
    total_decoded_samples: &mut u64,
) {
    let output_flat: &[f32] = unsafe {
        std::slice::from_raw_parts(output_stereo.as_ptr() as *const f32, output_frames * 2)
    };

    let mut pushed = 0;
    while pushed < output_flat.len() {
        match producer.push_slice(&output_flat[pushed..]) {
            n if n > 0 => {
                pushed += n;
                *total_decoded_samples += n as u64;
            }
            _ => {
                // Buffer is full - sleep briefly to avoid busy-wait
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        }
    }
}
