/* --- loonixtunesv2/src/audio/io/audiooutput.rs | audiooutput --- */
#[allow(non_snake_case)]
use crate::audio::dsp::{DspChain, DspProcessor};
use crate::audio::engine::OutputMode;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::traits::{Consumer, Observer};
use ringbuf::HeapCons;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
// #[allow(unused_imports)]
// use std::time::Duration;

pub enum AudioCommand {
    Play {
        should_stop: Arc<AtomicBool>,
        seek_mode: Arc<AtomicBool>,
        paused: Arc<AtomicBool>,
        flush_requested: Arc<AtomicBool>,
        seek_fade_remaining: Arc<AtomicU32>,
        volume_bits: Arc<AtomicU32>,
        balance_bits: Arc<AtomicU32>,
        mode: Arc<Mutex<OutputMode>>,
        dsp_chain: DspChain,
        dsp_enabled: Arc<AtomicBool>,
        normalizer_enabled: Arc<AtomicBool>,
        normalizer: Arc<Mutex<crate::audio::dsp::normalizer::AudioNormalizer>>,
        samples_played: Arc<AtomicU64>,
        empty_callback_count: Arc<AtomicU32>,
        output_state: Arc<AtomicU32>,
        decoder_eof: Arc<AtomicBool>,
        is_bluetooth_detected: Arc<AtomicBool>,
    },
    Stop,
    Flush,
    Exit,
}

#[derive(Debug, Clone)]
pub struct AudioDevice {
    pub name: String,
    pub description: String,
    pub index: u32,
}

fn f32_to_bits(f: f32) -> u32 {
    f.to_bits()
}

const BUFFER_EMPTY_THRESHOLD: u32 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputState {
    Priming,
    Running,
    Stopping,
}

impl Default for OutputState {
    fn default() -> Self {
        OutputState::Priming
    }
}

const OUTPUT_STATE_PRIMING: u32 = 0;
const OUTPUT_STATE_RUNNING: u32 = 1;
const OUTPUT_STATE_STOPPING: u32 = 2;
const OUTPUT_STATE_ERROR: u32 = 3;

pub struct AudioOutput {
    is_running: Arc<AtomicBool>,
    is_started: Arc<AtomicBool>,
    should_stop: Arc<AtomicBool>,
    volume_bits: Arc<AtomicU32>,
    balance_bits: Arc<AtomicU32>,
    mode_shared: Arc<Mutex<OutputMode>>,
    pub mode: OutputMode,
    command_tx: mpsc::Sender<AudioCommand>,
    thread_handle: Option<thread::JoinHandle<()>>,
    dsp_chain: DspChain,
    pub dsp_enabled: Arc<AtomicBool>,
    samples_played: Arc<AtomicU64>,
    sample_rate: u32,
    ring_buffer_capacity: usize,
    empty_callback_count: Arc<AtomicU32>,
    loop_reset: Arc<AtomicBool>,
    seek_fade_remaining: Arc<AtomicU32>,
    seek_mode: Arc<AtomicBool>,
    paused: Arc<AtomicBool>,
    flush_requested: Arc<AtomicBool>,
    resume_frame_counter: Arc<AtomicU32>,
    consumer: Arc<Mutex<Option<HeapCons<f32>>>>,
        normalizer_enabled: Arc<AtomicBool>,
        normalizer: Arc<Mutex<crate::audio::dsp::normalizer::AudioNormalizer>>,
        limiter_enabled: Arc<AtomicBool>,
        limiter: Arc<Mutex<crate::audio::dsp::limiter::Limiter>>,
        selected_device_index: Arc<Mutex<Option<usize>>>,
    is_bluetooth_detected: Arc<AtomicBool>,
    pub reconnecting: Arc<AtomicBool>,
    pub reconnect_attempts: Arc<AtomicU32>,
    current_device_name: Arc<Mutex<Option<String>>>,
    available_devices: Arc<Mutex<Vec<AudioDevice>>>,
    output_state: Arc<AtomicU32>,
    decoder_eof: Arc<AtomicBool>,
    stream: Arc<Mutex<Option<cpal::Stream>>>,
    stream_consumer: Arc<Mutex<Option<HeapCons<f32>>>>,
    stream_should_stop: Arc<AtomicBool>,
    stream_seek_mode: Arc<AtomicBool>,
    stream_paused: Arc<AtomicBool>,
    stream_flush: Arc<AtomicBool>,
    stream_seek_fade: Arc<AtomicU32>,
    stream_volume: Arc<AtomicU32>,
    stream_balance: Arc<AtomicU32>,
    stream_mode: Arc<Mutex<OutputMode>>,
    stream_dsp_chain: Arc<Mutex<DspChain>>,
    stream_dsp_enabled: Arc<AtomicBool>,
            stream_normalizer_enabled: Arc<AtomicBool>,
            stream_normalizer: Arc<Mutex<crate::audio::dsp::normalizer::AudioNormalizer>>,
            stream_limiter_enabled: Arc<AtomicBool>,
            stream_limiter: Arc<Mutex<crate::audio::dsp::limiter::Limiter>>,
            stream_samples_played: Arc<AtomicU64>,
    stream_empty_count: Arc<AtomicU32>,
    stream_output_state: Arc<AtomicU32>,
    stream_decoder_eof: Arc<AtomicBool>,
    stream_bt_detected: Arc<AtomicBool>,
}

impl Default for AudioOutput {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioOutput {
    pub fn new() -> Self {
        let (tx, _rx) = mpsc::channel();

        Self {
            is_running: Arc::new(AtomicBool::new(false)),
            is_started: Arc::new(AtomicBool::new(false)),
            should_stop: Arc::new(AtomicBool::new(false)),
            volume_bits: Arc::new(AtomicU32::new(f32_to_bits(1.0))),
            balance_bits: Arc::new(AtomicU32::new(f32_to_bits(0.0))),
            mode_shared: Arc::new(Mutex::new(OutputMode::Stereo)),
            mode: OutputMode::Stereo,
            command_tx: tx,
            thread_handle: None,
            dsp_chain: DspChain::default(),
            dsp_enabled: Arc::new(AtomicBool::new(true)),
            samples_played: Arc::new(AtomicU64::new(0)),
            sample_rate: 48000,
            ring_buffer_capacity: 0,
            empty_callback_count: Arc::new(AtomicU32::new(0)),
            loop_reset: Arc::new(AtomicBool::new(false)),
            seek_fade_remaining: Arc::new(AtomicU32::new(0)),
            seek_mode: Arc::new(AtomicBool::new(false)),
            paused: Arc::new(AtomicBool::new(false)),
            flush_requested: Arc::new(AtomicBool::new(false)),
            resume_frame_counter: Arc::new(AtomicU32::new(0)),
            consumer: Arc::new(Mutex::new(None)),
            normalizer_enabled: Arc::new(AtomicBool::new(false)),
             normalizer: Arc::new(Mutex::new(
                 crate::audio::dsp::normalizer::AudioNormalizer::new(true),
             )),
             limiter_enabled: Arc::new(AtomicBool::new(true)),
             limiter: Arc::new(Mutex::new(
                 crate::audio::dsp::limiter::Limiter::new(),
             )),
            selected_device_index: Arc::new(Mutex::new(None)),
            is_bluetooth_detected: Arc::new(AtomicBool::new(false)),
            reconnecting: Arc::new(AtomicBool::new(false)),
            reconnect_attempts: Arc::new(AtomicU32::new(0)),
            current_device_name: Arc::new(Mutex::new(None)),
            available_devices: Arc::new(Mutex::new(Vec::new())),
            output_state: Arc::new(AtomicU32::new(OUTPUT_STATE_PRIMING)),
            decoder_eof: Arc::new(AtomicBool::new(false)),
            stream: Arc::new(Mutex::new(None)),
            stream_consumer: Arc::new(Mutex::new(None)),
            stream_should_stop: Arc::new(AtomicBool::new(false)),
            stream_seek_mode: Arc::new(AtomicBool::new(false)),
            stream_paused: Arc::new(AtomicBool::new(false)),
            stream_flush: Arc::new(AtomicBool::new(false)),
            stream_seek_fade: Arc::new(AtomicU32::new(0)),
            stream_volume: Arc::new(AtomicU32::new(f32_to_bits(1.0))),
            stream_balance: Arc::new(AtomicU32::new(f32_to_bits(0.0))),
            stream_mode: Arc::new(Mutex::new(OutputMode::Stereo)),
            stream_dsp_chain: Arc::new(Mutex::new(DspChain::default())),
            stream_dsp_enabled: Arc::new(AtomicBool::new(true)),
            stream_normalizer_enabled: Arc::new(AtomicBool::new(false)),
             stream_normalizer: Arc::new(Mutex::new(
                 crate::audio::dsp::normalizer::AudioNormalizer::new(true),
             )),
             stream_limiter_enabled: Arc::new(AtomicBool::new(true)),
             stream_limiter: Arc::new(Mutex::new(
                 crate::audio::dsp::limiter::Limiter::new(),
             )),
            stream_samples_played: Arc::new(AtomicU64::new(0)),
            stream_empty_count: Arc::new(AtomicU32::new(0)),
            stream_output_state: Arc::new(AtomicU32::new(OUTPUT_STATE_PRIMING)),
            stream_decoder_eof: Arc::new(AtomicBool::new(false)),
            stream_bt_detected: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn request_loop_reset(&self) {
        self.loop_reset.store(true, Ordering::SeqCst);
    }

    pub fn get_dsp_chain(&self) -> DspChain {
        self.dsp_chain.clone()
    }

    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        if !self.is_started.load(Ordering::SeqCst) {
            self.sample_rate = sample_rate;
        }
    }

    pub fn get_sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn get_samples_played_arc(&self) -> Arc<AtomicU64> {
        self.samples_played.clone()
    }

    pub fn get_samples_played(&self) -> u64 {
        self.samples_played.load(Ordering::SeqCst)
    }

    pub fn set_samples_played(&self, samples: u64) {
        self.samples_played.store(samples, Ordering::SeqCst);
    }

    pub fn reset_samples_played(&self, samples: u64) {
        self.samples_played.store(samples, Ordering::SeqCst);
    }

    pub fn get_reconnecting_status(&self) -> bool {
        self.reconnecting.load(Ordering::Relaxed)
    }

    pub fn get_reconnect_attempts(&self) -> u32 {
        self.reconnect_attempts.load(Ordering::Relaxed)
    }

    pub fn force_reconnect(&self) {
        self.reconnecting.store(true, Ordering::Relaxed);
        self.reconnect_attempts.store(0, Ordering::Relaxed);
    }

    pub fn clear_buffer(&self) {
        self.flush_requested.store(true, Ordering::SeqCst);
        self.stream_flush.store(true, Ordering::SeqCst);
    }

    pub fn is_buffer_empty(&self) -> bool {
        if let Ok(cons) = self.consumer.lock() {
            if let Some(ref c) = *cons {
                return c.is_empty();
            }
        }
        true
    }

    pub fn is_ring_buffer_empty(&self) -> bool {
        self.is_buffer_empty()
    }

    pub fn is_truly_buffer_empty(&self) -> bool {
        self.empty_callback_count.load(Ordering::Relaxed) >= BUFFER_EMPTY_THRESHOLD
    }

    pub fn get_buffer_len(&self) -> usize {
        if let Ok(cons) = self.consumer.lock() {
            if let Some(ref c) = *cons {
                if !c.is_empty() {
                    return self.ring_buffer_capacity;
                }
                return 0;
            }
        }
        0
    }

    pub fn is_ring_buffer_ready(&self) -> bool {
        if let Ok(cons) = self.consumer.lock() {
            if let Some(ref c) = *cons {
                return !c.is_empty();
            }
        }
        false
    }

    pub fn is_seek_mode(&self) -> bool {
        self.seek_mode.load(Ordering::SeqCst)
    }

    pub fn reset_dsp(&self) {
        self.dsp_chain.reset();
    }

    pub fn update_dsp(&mut self, _settings: &crate::audio::dsp::DspSettings) {
        let rack = crate::audio::dsp::DspRack::build_rack(false);
        self.dsp_chain.swap_chain(rack);
    }

    pub fn update_mode_internal(&self) {
        if let Ok(mut m) = self.mode_shared.lock() {
            *m = self.mode;
        }
    }

    pub fn set_volume(&self, volume: f32) {
        self.volume_bits
            .store(f32_to_bits(volume), Ordering::SeqCst);
        self.stream_volume
            .store(f32_to_bits(volume), Ordering::SeqCst);
    }

    pub fn set_balance(&self, balance: f32) {
        self.balance_bits
            .store(f32_to_bits(balance), Ordering::SeqCst);
        self.stream_balance
            .store(f32_to_bits(balance), Ordering::SeqCst);
    }

    pub fn set_dsp_enabled(&self, enabled: bool) {
        self.dsp_enabled.store(enabled, Ordering::SeqCst);
        self.stream_dsp_enabled.store(enabled, Ordering::SeqCst);
    }

    pub fn is_dsp_enabled(&self) -> bool {
        self.dsp_enabled.load(Ordering::SeqCst)
    }

    pub fn set_normalizer_enabled(&mut self, enabled: bool) {
        self.normalizer_enabled.store(enabled, Ordering::SeqCst);
        self.stream_normalizer_enabled
            .store(enabled, Ordering::SeqCst);
    }

    pub fn set_normalizer_gain(&self, gain: f32) {
        if let Ok(mut norm) = self.normalizer.lock() {
            norm.set_fixed_gain(gain);
        }
        if let Ok(mut norm) = self.stream_normalizer.lock() {
            norm.set_fixed_gain(gain);
        }
    }

    pub fn get_normalizer_arc(&self) -> Arc<Mutex<crate::audio::dsp::normalizer::AudioNormalizer>> {
        self.normalizer.clone()
    }

    pub fn get_available_devices(&self) -> Vec<AudioDevice> {
        self.available_devices
            .lock()
            .ok()
            .map(|d| d.clone())
            .unwrap_or_default()
    }

    pub fn get_output_devices(&self) -> Vec<String> {
        if let Ok(devs) = self.available_devices.lock() {
            if !devs.is_empty() {
                return devs.iter().map(|d| d.description.clone()).collect();
            }
        }
        vec!["Default Output".to_string()]
    }

    pub fn set_output_device(&self, index: usize) {
        if let Ok(mut selected) = self.selected_device_index.lock() {
            *selected = Some(index);
        }
    }

    pub fn get_selected_device_index(&self) -> Option<usize> {
        self.selected_device_index
            .lock()
            .ok()
            .and_then(|guard| *guard)
    }

    pub fn select_device(&mut self, device_name: String) {
        let is_bluetooth = device_name.to_lowercase().contains("bluetooth");
        self.is_bluetooth_detected
            .store(is_bluetooth, Ordering::SeqCst);

        if let Ok(mut selected) = self.selected_device_index.lock() {
            *selected = Some(device_name.parse().unwrap_or(0));
        }
    }

    pub fn change_device(&self, _device_name: Option<String>) -> Result<(), String> {
        Ok(())
    }

    pub fn get_current_device_name(&self) -> Option<String> {
        self.current_device_name.lock().ok()?.clone()
    }

    pub fn set_output_state(&self, state: OutputState) {
        let state_val = match state {
            OutputState::Priming => OUTPUT_STATE_PRIMING,
            OutputState::Running => OUTPUT_STATE_RUNNING,
            OutputState::Stopping => OUTPUT_STATE_STOPPING,
        };
        self.output_state.store(state_val, Ordering::SeqCst);
        self.stream_output_state.store(state_val, Ordering::SeqCst);
    }

    pub fn get_output_state(&self) -> OutputState {
        match self.output_state.load(Ordering::SeqCst) {
            OUTPUT_STATE_PRIMING => OutputState::Priming,
            OUTPUT_STATE_RUNNING => OutputState::Running,
            OUTPUT_STATE_STOPPING => OutputState::Stopping,
            _ => OutputState::Priming,
        }
    }

    pub fn set_decoder_eof(&self, eof: bool) {
        self.decoder_eof.store(eof, Ordering::SeqCst);
        self.stream_decoder_eof.store(eof, Ordering::SeqCst);
    }

    fn enumerate_devices() -> Vec<AudioDevice> {
        let mut devices = Vec::new();

        let host = cpal::default_host();

        if let Ok(device_list) = host.output_devices() {
            for (idx, device) in device_list.enumerate() {
                let name = device.name().unwrap_or_else(|_| "Unknown".to_string());
                devices.push(AudioDevice {
                    name: name.clone(),
                    description: name,
                    index: idx as u32,
                });
            }
        }

        devices
    }

    pub fn start(&mut self, consumer: HeapCons<f32>, clear_old: bool, buffer_capacity: usize) {
        if clear_old {
            self.seek_fade_remaining.store(0, Ordering::SeqCst);
            self.stream_seek_fade.store(0, Ordering::SeqCst);
        }

        self.ring_buffer_capacity = buffer_capacity;
        self.available_devices = Arc::new(Mutex::new(Self::enumerate_devices()));

        // Store consumer
        if let Ok(mut c) = self.consumer.lock() {
            *c = Some(consumer);
        }

        // Update stream shared state
        self.stream_should_stop = self.should_stop.clone();
        self.stream_seek_mode = self.seek_mode.clone();
        self.stream_paused = self.paused.clone();
        self.stream_flush = self.flush_requested.clone();
        self.stream_seek_fade = self.seek_fade_remaining.clone();
        self.stream_volume = self.volume_bits.clone();
        self.stream_balance = self.balance_bits.clone();
        self.stream_mode = self.mode_shared.clone();
        self.stream_dsp_chain = Arc::new(Mutex::new(self.dsp_chain.clone()));
        self.stream_dsp_enabled = self.dsp_enabled.clone();
        self.stream_normalizer_enabled = self.normalizer_enabled.clone();
        self.stream_normalizer = self.normalizer.clone();
        self.stream_limiter_enabled = self.limiter_enabled.clone();
        self.stream_limiter = self.limiter.clone();
        self.stream_samples_played = self.samples_played.clone();
        self.stream_empty_count = self.empty_callback_count.clone();
        self.stream_output_state = self.output_state.clone();
        self.stream_decoder_eof = self.decoder_eof.clone();
        self.stream_bt_detected = self.is_bluetooth_detected.clone();

        if let Ok(mut c) = self.stream_consumer.lock() {
            // Caching is not Clone - use take() to move ownership
            if let Ok(mut main_c) = self.consumer.lock() {
                *c = main_c.take();
            }
        }

        // Create CPAL stream
        match self.create_cpal_stream(self.sample_rate) {
            Ok(stream) => {
                self.should_stop.store(false, Ordering::SeqCst);
                self.is_running.store(true, Ordering::SeqCst);
                self.output_state
                    .store(OUTPUT_STATE_PRIMING, Ordering::SeqCst);
                self.decoder_eof.store(false, Ordering::SeqCst);
                self.empty_callback_count.store(0, Ordering::Relaxed);

                if let Ok(mut s) = self.stream.lock() {
                    *s = Some(stream);
                }

                self.is_started.store(true, Ordering::SeqCst);
            }
            Err(e) => {
                eprintln!("[AudioOutput] Failed to create CPAL stream: {}", e);
            }
        }

        self.paused.store(false, Ordering::SeqCst);
    }

    fn create_cpal_stream(&self, sample_rate: u32) -> Result<cpal::Stream, String> {
        let host = cpal::default_host();

        let device = host
            .default_output_device()
            .ok_or_else(|| "No default output device".to_string())?;

        // Query supported config to branch on sample format without forcing a rate
        // Use device-provided config to avoid resampling artefacts
        let supported_config = device
            .default_output_config()
            .map_err(|e| format!("Failed to get default output config: {}", e))?;
        // Debug: print device sample format as reported by CPAL/WASAPI/etc.
        let sample_format = {
            let fmt = supported_config.sample_format();
            println!("{:?}", fmt);
            fmt
        };
        // Capture the device's actual rate for possible use by the decoder
        let device_rate = supported_config.sample_rate().0;
        // Expose device_rate to downstream components (decoder) if needed.
        let _device_rate_for_decoder = device_rate;
        let config: cpal::StreamConfig = supported_config.into();

        let channels = config.channels as usize;

        // Capture shared state for callback
        let consumer = self.stream_consumer.clone();
        let should_stop = self.stream_should_stop.clone();
        let seek_mode = self.stream_seek_mode.clone();
        let paused = self.stream_paused.clone();
        let flush = self.stream_flush.clone();
        let seek_fade = self.stream_seek_fade.clone();
        let volume = self.stream_volume.clone();
        let balance = self.stream_balance.clone();
        let mode = self.stream_mode.clone();
        let dsp_chain = self.stream_dsp_chain.clone();
        let dsp_enabled = self.stream_dsp_enabled.clone();
        let normalizer_enabled = self.stream_normalizer_enabled.clone();
        let normalizer = self.stream_normalizer.clone();
        let limiter_enabled = self.stream_limiter_enabled.clone();
        let limiter = self.stream_limiter.clone();
        let samples_played = self.stream_samples_played.clone();
        let empty_count = self.stream_empty_count.clone();
        let output_state = self.stream_output_state.clone();
        let decoder_eof = self.stream_decoder_eof.clone();
        let bt_detected = self.stream_bt_detected.clone();

        match sample_format {
            cpal::SampleFormat::F32 => {
                let stream = device
                    .build_output_stream(
                        &config,
                        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                            let samples_per_write = data.len();
                            let channels = config.channels as usize;
                            let frames = samples_per_write / channels;

                            // Get consumer
                            let mut read_buffer = vec![0.0f32; samples_per_write];
                            let mut processed_buffer = vec![0.0f32; samples_per_write];
                            let mut norm_input = vec![0.0f32; samples_per_write];

                            let is_stopped = should_stop.load(Ordering::SeqCst);
                            if is_stopped {
                                data.fill(0.0);
                                return;
                            }

                            let is_seeking = seek_mode.load(Ordering::SeqCst);
                            let is_paused = paused.load(Ordering::SeqCst);
                            let do_flush = flush.load(Ordering::SeqCst);

                            if do_flush {
                                if let Ok(mut c) = consumer.lock() {
                                    if let Some(ref mut cons) = *c {
                                        loop {
                                            let drained = cons.pop_slice(&mut read_buffer);
                                            if drained == 0 {
                                                break;
                                            }
                                        }
                                    }
                                }
                                flush.store(false, Ordering::SeqCst);
                                empty_count.store(0, Ordering::SeqCst);
                            }

                            if is_seeking || is_paused {
                                read_buffer.fill(0.0);
                            } else {
                                if let Ok(mut c) = consumer.lock() {
                                    if let Some(ref mut cons) = *c {
                                        let samples_read = cons.pop_slice(&mut read_buffer);
                                        if samples_read == 0 {
                                            empty_count.fetch_add(1, Ordering::Relaxed);
                                        } else {
                                            empty_count.store(0, Ordering::Relaxed);
                                        }
                                    }
                                }
                            }

                            // DSP processing
                            let state = output_state.load(Ordering::SeqCst);

                            if state == OUTPUT_STATE_PRIMING && read_buffer.iter().any(|&x| x != 0.0) {
                                output_state.store(OUTPUT_STATE_RUNNING, Ordering::SeqCst);
                            }

                            // ---- NORMALIZER (Engine Core - Always runs first) ----
                            if normalizer_enabled.load(Ordering::SeqCst) {
                                if let Ok(mut norm) = normalizer.try_lock() {
                                    norm.process(&read_buffer[..samples_per_write], &mut processed_buffer[..samples_per_write]);
                                }
                            } else {
                                processed_buffer[..samples_per_write].copy_from_slice(&read_buffer[..samples_per_write]);
                            }

                            // ---- DSP RACK (Toggleable) ----
                            if dsp_enabled.load(Ordering::SeqCst) {
                                if let Ok(chain) = dsp_chain.lock() {
                                    // Use temp buffer to avoid borrow conflict
                                    norm_input[..samples_per_write]
                                        .copy_from_slice(&processed_buffer[..samples_per_write]);
                                    chain.process(&norm_input[..samples_per_write], &mut processed_buffer[..samples_per_write]);
                                }
                            }

                            // ---- LIMITER (Engine Core - Always runs after DSP) ----
                            if limiter_enabled.load(Ordering::SeqCst) {
                                if let Ok(mut limiter) = limiter.try_lock() {
                                    // Use temp buffer to avoid borrow conflict
                                    norm_input[..samples_per_write]
                                        .copy_from_slice(&processed_buffer[..samples_per_write]);
                                    limiter.process(&norm_input[..samples_per_write], &mut processed_buffer[..samples_per_write]);
                                }
                            }

                            // Volume and balance
                            let vol = f32::from_bits(volume.load(Ordering::Relaxed));
                            let bal = f32::from_bits(balance.load(Ordering::Relaxed));
                            let left_gain = if bal > 0.0 { 1.0 - bal } else { 1.0 };
                            let right_gain = if bal < 0.0 { 1.0 + bal } else { 1.0 };

                            let current_mode = *mode.lock().unwrap_or_else(|e| e.into_inner());

                            for frame in 0..frames {
                                let mut left = processed_buffer[frame * 2];
                                let mut right = processed_buffer[frame * 2 + 1];

                                left *= left_gain;
                                right *= right_gain;

                                match current_mode {
                                    OutputMode::Mono => {
                                        let mono = (left + right) * 0.5;
                                        left = mono;
                                        right = mono;
                                    }
                                    OutputMode::Surround => {
                                        let diff = (left - right) * 0.3;
                                        left += diff;
                                        right -= diff;
                                    }
                                    OutputMode::Stereo => {}
                                }

                                left *= vol;
                                right *= vol;

                                if !left.is_finite() {
                                    left = 0.0;
                                }
                                if !right.is_finite() {
                                    right = 0.0;
                                }
                                left = left.clamp(-0.99, 0.99);
                                right = right.clamp(-0.99, 0.99);

                                processed_buffer[frame * 2] = left;
                                processed_buffer[frame * 2 + 1] = right;
                            }

                            // Write to output (F32 path: write raw processed buffer)
                            data.copy_from_slice(&processed_buffer);

                            // Update samples played
                            samples_played.fetch_add(frames as u64, Ordering::SeqCst);
                        },
                        |err| eprintln!("[AudioOutput] Stream error: {}", err),
                        None,
                    )
                    .map_err(|e| format!("Failed to build output stream: {}", e))?;

                stream
                    .play()
                    .map_err(|e| format!("Failed to play stream: {}", e))?;

                Ok(stream)
            }
            _ => { return Err(format!("Unsupported sample format: {:?}", sample_format)); }
            cpal::SampleFormat::I16 => {
                let stream = device
                    .build_output_stream(
                        &config,
                        move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                            // For brevity, reuse the F32 processing path and cast to i16 at write
                            // This mirrors the F32 path but outputs 16-bit signed samples
                            let samples_per_write = data.len();
                            let channels = config.channels as usize;
                            let frames = samples_per_write / channels;

                            // Buffers in f32 for processing
                            let mut read_buffer = vec![0.0f32; samples_per_write];
                            let mut processed_buffer = vec![0.0f32; samples_per_write];
                            let mut norm_input = vec![0.0f32; samples_per_write];

                            let is_stopped = should_stop.load(Ordering::SeqCst);
                            if is_stopped {
                                for i in 0..samples_per_write {
                                    data[i] = 0;
                                }
                                return;
                            }

                            let is_seeking = seek_mode.load(Ordering::SeqCst);
                            let is_paused = paused.load(Ordering::SeqCst);
                            let do_flush = flush.load(Ordering::SeqCst);

                            if do_flush {
                                if let Ok(mut c) = consumer.lock() {
                                    if let Some(ref mut cons) = *c {
                                        loop {
                                            let drained = cons.pop_slice(&mut read_buffer);
                                            if drained == 0 {
                                                break;
                                            }
                                        }
                                    }
                                }
                                flush.store(false, Ordering::SeqCst);
                                empty_count.store(0, Ordering::SeqCst);
                            }

                            if is_seeking || is_paused {
                                read_buffer.fill(0.0);
                            } else {
                                if let Ok(mut c) = consumer.lock() {
                                    if let Some(ref mut cons) = *c {
                                        let samples_read = cons.pop_slice(&mut read_buffer);
                                        if samples_read == 0 {
                                            empty_count.fetch_add(1, Ordering::Relaxed);
                                        } else {
                                            empty_count.store(0, Ordering::Relaxed);
                                        }
                                    }
                                }
                            }

                            // DSP processing
                            let state = output_state.load(Ordering::SeqCst);
                            let is_eof = decoder_eof.load(Ordering::SeqCst);

                            if state == OUTPUT_STATE_PRIMING && read_buffer.iter().any(|&x| x != 0.0) {
                                output_state.store(OUTPUT_STATE_RUNNING, Ordering::SeqCst);
                            }

                            // ---- NORMALIZER (Engine Core - Always runs first) ----
                            if normalizer_enabled.load(Ordering::SeqCst) {
                                if let Ok(mut norm) = normalizer.try_lock() {
                                    norm.process(&read_buffer[..samples_per_write], &mut processed_buffer[..samples_per_write]);
                                }
                            } else {
                                processed_buffer[..samples_per_write].copy_from_slice(&read_buffer[..samples_per_write]);
                            }

                            // ---- DSP RACK (Toggleable) ----
                            if dsp_enabled.load(Ordering::SeqCst) {
                                if let Ok(chain) = dsp_chain.lock() {
                                    // Use temp buffer to avoid borrow conflict
                                    norm_input[..samples_per_write]
                                        .copy_from_slice(&processed_buffer[..samples_per_write]);
                                    chain.process(&norm_input[..samples_per_write], &mut processed_buffer[..samples_per_write]);
                                }
                            }

                            // ---- LIMITER (Engine Core - Always runs after DSP) ----
                            if limiter_enabled.load(Ordering::SeqCst) {
                                if let Ok(mut limiter) = limiter.try_lock() {
                                    // Use temp buffer to avoid borrow conflict
                                    norm_input[..samples_per_write]
                                        .copy_from_slice(&processed_buffer[..samples_per_write]);
                                    limiter.process(&norm_input[..samples_per_write], &mut processed_buffer[..samples_per_write]);
                                }
                            }

                            // Volume and balance
                            let vol = f32::from_bits(volume.load(Ordering::Relaxed));
                            let bal = f32::from_bits(balance.load(Ordering::Relaxed));
                            let left_gain = if bal > 0.0 { 1.0 - bal } else { 1.0 };
                            let right_gain = if bal < 0.0 { 1.0 + bal } else { 1.0 };

                            let current_mode = *mode.lock().unwrap_or_else(|e| e.into_inner());

                            for frame in 0..frames {
                                let mut left = processed_buffer[frame * 2];
                                let mut right = processed_buffer[frame * 2 + 1];

                                left *= left_gain;
                                right *= right_gain;

                                match current_mode {
                                    OutputMode::Mono => {
                                        let mono = (left + right) * 0.5;
                                        left = mono;
                                        right = mono;
                                    }
                                    OutputMode::Surround => {
                                        let diff = (left - right) * 0.3;
                                        left += diff;
                                        right -= diff;
                                    }
                                    OutputMode::Stereo => {}
                                }

                                left *= vol;
                                right *= vol;

                                if !left.is_finite() {
                                    left = 0.0;
                                }
                                if !right.is_finite() {
                                    right = 0.0;
                                }
                                left = left.clamp(-0.99, 0.99);
                                right = right.clamp(-0.99, 0.99);

                                processed_buffer[frame * 2] = left;
                                processed_buffer[frame * 2 + 1] = right;
                            }

                            for i in 0..samples_per_write {
                                data[i] = (processed_buffer[i] * 32767.0) as i16;
                            }

                            samples_played.fetch_add(frames as u64, Ordering::SeqCst);
                        },
                        |err| eprintln!("[AudioOutput] Stream error: {}", err),
                        None,
                    )
                    .map_err(|e| format!("Failed to build output stream: {}", e))?;

                stream
                    .play()
                    .map_err(|e| format!("Failed to play stream: {}", e))?;

                Ok(stream)
            }
        }
    }

    pub fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
        self.should_stop.store(true, Ordering::SeqCst);
        self.seek_mode.store(false, Ordering::SeqCst);
        self.seek_fade_remaining.store(0, Ordering::SeqCst);
        self.resume_frame_counter.store(0, Ordering::SeqCst);
        self.paused.store(false, Ordering::SeqCst);
        self.output_state
            .store(OUTPUT_STATE_STOPPING, Ordering::SeqCst);

        if let Ok(mut s) = self.stream.lock() {
            if let Some(ref stream) = *s {
                let _ = stream.pause();
            }
            *s = None;
        }

        self.reset_dsp();
    }

    pub fn start_consumers(&self) {
        self.is_running.store(true, Ordering::SeqCst);
    }

    pub fn pause(&mut self) {
        self.paused.store(true, Ordering::SeqCst);
    }

    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::SeqCst)
    }

    pub fn trigger_seek_fade(&self) {
        let fade_samples = (self.sample_rate as f32 * 0.015) as u32;
        self.seek_fade_remaining
            .store(fade_samples, Ordering::SeqCst);
    }

    pub fn set_seek_mode(&self, seeking: bool) {
        self.seek_mode.store(seeking, Ordering::SeqCst);
    }

    pub fn trigger_delayed_resume(&self) {
        self.resume_frame_counter.store(2, Ordering::SeqCst);
    }

    pub fn check_resume_counter(&self) -> bool {
        let remaining = self.resume_frame_counter.load(Ordering::SeqCst);
        if remaining > 0 {
            self.resume_frame_counter
                .store(remaining - 1, Ordering::SeqCst);
            return false;
        }
        true
    }

    pub fn resume(&mut self) {
        self.paused.store(false, Ordering::SeqCst);
        self.is_running.store(true, Ordering::SeqCst);
    }
}

impl Drop for AudioOutput {
    fn drop(&mut self) {
        self.is_running.store(false, Ordering::SeqCst);
        self.is_started.store(false, Ordering::SeqCst);

        let _ = self.command_tx.send(AudioCommand::Exit);

        if let Ok(mut s) = self.stream.lock() {
            if let Some(ref stream) = *s {
                let _ = stream.pause();
            }
            *s = None;
        }

        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}
