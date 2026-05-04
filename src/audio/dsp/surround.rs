/* --- loonixtunesv2/src/audio/dsp/surround.rs | surround --- */

use crate::audio::dsp::DspProcessor;
use crate::audio::samplerate; // Import sample rate module
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::OnceLock;

static SURROUND_ENABLED: OnceLock<AtomicBool> = OnceLock::new();
static SURROUND_WIDTH: OnceLock<AtomicU32> = OnceLock::new();
static SURROUND_BASS_SAFE: OnceLock<AtomicU32> = OnceLock::new();
static SURROUND_MAGIC_MODE: OnceLock<AtomicBool> = OnceLock::new();

pub fn get_surround_enabled_arc() -> &'static AtomicBool {
    SURROUND_ENABLED.get_or_init(|| AtomicBool::new(false))
}

pub fn get_surround_width_arc() -> &'static AtomicU32 {
    SURROUND_WIDTH.get_or_init(|| AtomicU32::new(1.0_f32.to_bits()))
}

pub fn get_surround_bass_safe_arc() -> &'static AtomicU32 {
    SURROUND_BASS_SAFE.get_or_init(|| AtomicU32::new(1.0_f32.to_bits()))
}

pub fn get_surround_magic_mode_arc() -> &'static AtomicBool {
    SURROUND_MAGIC_MODE.get_or_init(|| AtomicBool::new(false))
}

fn bits_to_f32(bits: u32) -> f32 {
    f32::from_bits(bits)
}

pub struct SurroundProcessor {
    current_width: f32,
    target_width: f32,

    hp_prev_in: f32,
    hp_prev_out: f32,
    hp_coeff: f32,

    smoothing_coeff: f32,
}

impl SurroundProcessor {
    pub fn new() -> Self {
        Self {
            current_width: 1.0,
            target_width: 1.0,
            hp_prev_in: 0.0,
            hp_prev_out: 0.0,
            hp_coeff: 0.0,
            smoothing_coeff: 0.0,
        }
    }

    fn high_pass(&mut self, sample: f32) -> f32 {
        let out = self.hp_coeff * (self.hp_prev_out + sample - self.hp_prev_in);
        self.hp_prev_in = sample;
        self.hp_prev_out = out;
        out
    }

    fn smooth_width(&mut self) {
        self.current_width = self.current_width * self.smoothing_coeff
            + self.target_width * (1.0 - self.smoothing_coeff);
    }
}

impl DspProcessor for SurroundProcessor {
    fn process(&mut self, input: &[f32], output: &mut [f32]) {
        let is_on = get_surround_enabled_arc().load(Ordering::Relaxed);
        let raw_width = bits_to_f32(get_surround_width_arc().load(Ordering::Relaxed));
        let bass_safe = bits_to_f32(get_surround_bass_safe_arc().load(Ordering::Relaxed));

        if !is_on {
            // FIX: Pastikan copy benar dengan bounds check
            if output.len() >= input.len() {
                output[..input.len()].copy_from_slice(input);
            }
            return;
        }

        // Check if sample rate changed
        let rate_changed = samplerate::consume_rate_changed();
        if rate_changed {
            let rate = samplerate::get_rate();
            if rate > 0.0 {
                // Bass protection cutoff (preserve drum thump - 60Hz allows fundamental bass frequencies)
                let hp_cutoff = 60.0;
                let rc = 1.0 / (2.0 * std::f32::consts::PI * hp_cutoff);
                let dt = 1.0 / rate;
                self.hp_coeff = rc / (rc + dt);

                // 5ms smoothing
                let smoothing_time = 0.005;
                self.smoothing_coeff = (-1.0 / (rate * smoothing_time)).exp();
            }
        }

        // Soft-limit width supaya tidak brutal
        self.target_width = raw_width.clamp(0.0, 1.5);

        let len = input.len();

        for i in (0..len).step_by(2) {
            if i + 1 >= len {
                output[i] = input[i];
                break;
            }

            self.smooth_width();

            let left = input[i];
            let right = input[i + 1];

            let mid = (left + right) * 0.5;
            let mut side = (left - right) * 0.5;

            // Bass-safe high pass on side
            if bass_safe > 0.5 {
                side = self.high_pass(side);
            }

            // Controlled widening (lebih natural)
            let widened_side = side * self.current_width;

            let mut l = mid + widened_side;
            let mut r = mid - widened_side;

            // RMS-style normalization (lebih smooth dari norm sederhana)
            let energy = (l * l + r * r).sqrt();
            if energy > 1.0 {
                let inv = 1.0 / energy;
                l *= inv;
                r *= inv;
            }

            output[i] = l.clamp(-1.0, 1.0);
            output[i + 1] = r.clamp(-1.0, 1.0);
        }
    }

    fn reset(&mut self) {
        self.hp_prev_in = 0.0;
        self.hp_prev_out = 0.0;
        self.current_width = 1.0;
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn as_any_ref(&self) -> &dyn std::any::Any {
        self
    }
}
