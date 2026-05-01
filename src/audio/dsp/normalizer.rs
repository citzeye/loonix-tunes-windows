/* --- loonixtunesv2/src/audio/dsp/normalizer.rs | normalizer --- */
use crate::audio::dsp::DspProcessor;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, OnceLock};

static NORMALIZER_ENABLED: OnceLock<Arc<AtomicBool>> = OnceLock::new();
static NORMALIZER_SMOOTHING: OnceLock<Arc<AtomicU32>> = OnceLock::new();
static NORMALIZER_GAIN: OnceLock<Arc<AtomicU32>> = OnceLock::new();

pub fn get_enabled_arc() -> Arc<AtomicBool> {
    NORMALIZER_ENABLED
        .get_or_init(|| Arc::new(AtomicBool::new(true)))
        .clone()
}

fn get_smoothing_arc() -> Arc<AtomicU32> {
    NORMALIZER_SMOOTHING
        .get_or_init(|| Arc::new(AtomicU32::new(0.0015_f32.to_bits())))
        .clone()
}

pub fn get_normalizer_gain_arc() -> Arc<AtomicU32> {
    NORMALIZER_GAIN
        .get_or_init(|| Arc::new(AtomicU32::new(1.0_f32.to_bits())))
        .clone()
}

fn get_smoothing_value() -> f32 {
    f32::from_bits(get_smoothing_arc().load(Ordering::Relaxed))
}

pub fn get_normalizer_smoothing_arc() -> Arc<AtomicU32> {
    get_smoothing_arc()
}

/// Smoothing presets for cross-track transition speed.
pub enum SmoothingPreset {
    Slow,     // ~1.5s transition
    Balanced, // ~0.8s transition
    Fast,     // ~0.3s transition
}

impl SmoothingPreset {
    pub fn to_factor(&self) -> f32 {
        match self {
            SmoothingPreset::Slow => 0.001,
            SmoothingPreset::Balanced => 0.002,
            SmoothingPreset::Fast => 0.005,
        }
    }

    pub fn from_factor(f: f32) -> &'static str {
        if f <= 0.0015 {
            "Slow"
        } else if f <= 0.0035 {
            "Balanced"
        } else {
            "Fast"
        }
    }
}

/// Track-level gain applicator.
/// Applies precomputed RMS gain from scanner.
/// Not a compressor. Not a limiter.
pub struct AudioNormalizer {
    fixed_gain: f32,
    current_gain: f32,
}

unsafe impl Send for AudioNormalizer {}
unsafe impl Sync for AudioNormalizer {}

impl AudioNormalizer {
    pub fn new(enabled: bool) -> Self {
        get_enabled_arc().store(enabled, Ordering::SeqCst);
        Self {
            fixed_gain: 1.0,
            current_gain: 1.0,
        }
    }

    pub fn set_fixed_gain(&mut self, gain: f32) {
        self.fixed_gain = gain.clamp(0.01, 3.98);
    }

    pub fn get_fixed_gain(&self) -> f32 {
        self.fixed_gain
    }

    pub fn get_current_gain(&self) -> f32 {
        self.current_gain
    }

    pub fn snap_to_target(&mut self) {
        self.current_gain = self.fixed_gain;
    }
}

impl DspProcessor for AudioNormalizer {
    #[inline(always)]
    fn process(&mut self, input: &[f32], output: &mut [f32]) {
        let enabled = get_enabled_arc().load(Ordering::Relaxed);
        if !enabled {
            output.copy_from_slice(input);
            return;
        }

        let smoothing = get_smoothing_value();
        let target = self.fixed_gain;

        for i in 0..input.len() {
            // Simple smoothing towards fixed gain (no transient detection)
            self.current_gain += (target - self.current_gain) * smoothing;

            // Apply gain first, then surgical clip
            let amplified = input[i] * self.current_gain;
            output[i] = soft_clip(amplified);
        }
    }

    fn reset(&mut self) {
        self.current_gain = 1.0;
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn as_any_ref(&self) -> &dyn std::any::Any {
        self
    }
}

/// Safety clipper to prevent overshoot after normalization.
// Should rarely engage if scanner peak constraint works correctly.
#[inline(always)]
fn soft_clip(sample: f32) -> f32 {
    let threshold = 0.95;
    let abs_s = sample.abs();
    if abs_s <= threshold {
        sample // Bit-perfect transparency for transients
    } else {
        // Cubic clipping (more transparent than tanh for drums)
        let sign = sample.signum();
        let normalized = (abs_s - threshold) / (1.0 - threshold);
        let clipped = threshold + (1.0 - threshold) * (normalized - (normalized.powi(3) / 3.0));
        sign * clipped.clamp(0.0, 0.99)
    }
}
