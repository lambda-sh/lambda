#![allow(clippy::needless_return)]

//! Single-sound playback and transport controls.
//!
//! This module provides a minimal, backend-agnostic playback facade that
//! supports one active `SoundBuffer` at a time.

use std::{
  cell::UnsafeCell,
  mem::MaybeUninit,
  sync::{
    atomic::{
      AtomicU64,
      AtomicU8,
      AtomicUsize,
      Ordering,
    },
    Arc,
  },
};

use crate::audio::{
  AudioError,
  AudioOutputDevice,
  AudioOutputDeviceBuilder,
  AudioOutputWriter,
  SoundBuffer,
};

const DEFAULT_GAIN_RAMP_FRAMES: usize = 128;
const DEFAULT_OUTPUT_SAMPLE_RATE: u32 = 48_000;
const DEFAULT_OUTPUT_CHANNELS: u16 = 2;
const MAX_PLAYBACK_CHANNELS: usize = 8;
const PLAYBACK_COMMAND_CAPACITY: usize = 256;

/// A fixed-capacity, single-producer/single-consumer queue.
///
/// The queue is designed for real-time audio callbacks:
/// - `push` and `pop` MUST NOT block.
/// - `pop` MUST NOT allocate.
///
/// # Safety
/// This type is only sound when used as SPSC (exactly one producer thread and
/// one consumer thread).
#[allow(dead_code)]
struct CommandQueue<T, const CAPACITY: usize> {
  buffer: [UnsafeCell<MaybeUninit<T>>; CAPACITY],
  head: AtomicUsize,
  tail: AtomicUsize,
}

unsafe impl<T: Send, const CAPACITY: usize> Send for CommandQueue<T, CAPACITY> {}
unsafe impl<T: Send, const CAPACITY: usize> Sync for CommandQueue<T, CAPACITY> {}

#[allow(dead_code)]
impl<T, const CAPACITY: usize> CommandQueue<T, CAPACITY> {
  /// Create a new empty queue.
  ///
  /// # Returns
  /// A queue with a fixed capacity.
  fn new() -> Self {
    assert!(CAPACITY > 0, "command queue capacity must be non-zero");

    return Self {
      buffer: std::array::from_fn(|_| {
        return UnsafeCell::new(MaybeUninit::uninit());
      }),
      head: AtomicUsize::new(0),
      tail: AtomicUsize::new(0),
    };
  }

  /// Attempt to enqueue a value.
  ///
  /// # Arguments
  /// - `value`: The value to enqueue.
  ///
  /// # Returns
  /// `Ok(())` when the value was enqueued. `Err(value)` when the queue is full.
  fn push(&self, value: T) -> Result<(), T> {
    let head = self.head.load(Ordering::Acquire);
    let tail = self.tail.load(Ordering::Relaxed);

    if tail.wrapping_sub(head) >= CAPACITY {
      return Err(value);
    }

    let index = tail % CAPACITY;
    let slot = self.buffer[index].get();
    unsafe {
      (&mut *slot).write(value);
    }

    self.tail.store(tail.wrapping_add(1), Ordering::Release);
    return Ok(());
  }

  /// Attempt to dequeue a value.
  ///
  /// # Returns
  /// `Some(value)` when a value is available, otherwise `None`.
  fn pop(&self) -> Option<T> {
    let tail = self.tail.load(Ordering::Acquire);
    let head = self.head.load(Ordering::Relaxed);

    if head == tail {
      return None;
    }

    let index = head % CAPACITY;
    let slot = self.buffer[index].get();
    let value = unsafe { (&*slot).assume_init_read() };

    self.head.store(head.wrapping_add(1), Ordering::Release);
    return Some(value);
  }
}

impl<T, const CAPACITY: usize> Drop for CommandQueue<T, CAPACITY> {
  fn drop(&mut self) {
    let tail = self.tail.load(Ordering::Relaxed);
    let mut head = self.head.load(Ordering::Relaxed);

    while head != tail {
      let index = head % CAPACITY;
      let slot = self.buffer[index].get();
      unsafe {
        std::ptr::drop_in_place((&mut *slot).as_mut_ptr());
      }
      head = head.wrapping_add(1);
    }

    return;
  }
}

/// A linear gain ramp used to de-click transport transitions.
#[derive(Clone, Copy, Debug, PartialEq)]
struct GainRamp {
  current: f32,
  target: f32,
  step: f32,
  frames_remaining: usize,
}

impl GainRamp {
  /// Create a silent ramp with a target gain of `0.0`.
  ///
  /// # Returns
  /// A `GainRamp` initialized to silence.
  fn silent() -> Self {
    return Self {
      current: 0.0,
      target: 0.0,
      step: 0.0,
      frames_remaining: 0,
    };
  }

  /// Begin ramping the gain toward a target.
  ///
  /// # Arguments
  /// - `target`: Target gain in nominal range `[0.0, 1.0]`.
  /// - `frames`: Ramp duration in frames. When `0`, the gain changes
  ///   immediately.
  ///
  /// # Returns
  /// `()` after updating the ramp parameters.
  fn start(&mut self, target: f32, frames: usize) {
    let target = target.clamp(0.0, 1.0);

    if frames == 0 || (self.current - target).abs() <= f32::EPSILON {
      self.current = target;
      self.target = target;
      self.step = 0.0;
      self.frames_remaining = 0;
      return;
    }

    self.target = target;
    self.frames_remaining = frames;
    self.step = (target - self.current) / frames as f32;
    return;
  }

  /// Return whether the ramp is fully silent and stable.
  ///
  /// # Returns
  /// `true` if the current and target gain are both `0.0` with no remaining
  /// ramp frames.
  fn is_silent(&self) -> bool {
    return self.frames_remaining == 0
      && self.current.abs() <= f32::EPSILON
      && self.target.abs() <= f32::EPSILON;
  }

  /// Advance the ramp by one output frame.
  ///
  /// # Returns
  /// `()` after advancing the ramp state.
  fn advance_frame(&mut self) {
    if self.frames_remaining == 0 {
      return;
    }

    self.current += self.step;
    self.frames_remaining = self.frames_remaining.saturating_sub(1);

    if self.frames_remaining == 0 {
      self.current = self.target;
      self.step = 0.0;
    }

    return;
  }
}

/// Deterministic single-slot playback scheduler.
///
/// This scheduler is designed to run inside a real-time audio callback and
/// MUST NOT allocate or block while rendering audio.
#[allow(dead_code)]
struct PlaybackScheduler {
  state: PlaybackState,
  looping: bool,
  cursor_samples: usize,
  channels: usize,
  ramp_frames: usize,
  gain: GainRamp,
  buffer: Option<Arc<SoundBuffer>>,
  last_frame_samples: [f32; MAX_PLAYBACK_CHANNELS],
}

#[allow(dead_code)]
impl PlaybackScheduler {
  /// Create a scheduler configured for a fixed output channel count.
  ///
  /// # Arguments
  /// - `channels`: Interleaved output channel count.
  ///
  /// # Returns
  /// A scheduler initialized to `Stopped` with no buffer.
  fn new(channels: usize) -> Self {
    return Self::new_with_ramp_frames(channels, DEFAULT_GAIN_RAMP_FRAMES);
  }

  /// Create a scheduler configured for a fixed output channel count and ramp.
  ///
  /// # Arguments
  /// - `channels`: Interleaved output channel count.
  /// - `ramp_frames`: Gain ramp length for transport de-clicking in frames.
  ///
  /// # Returns
  /// A scheduler initialized to `Stopped` with no buffer.
  fn new_with_ramp_frames(channels: usize, ramp_frames: usize) -> Self {
    return Self {
      state: PlaybackState::Stopped,
      looping: false,
      cursor_samples: 0,
      channels,
      ramp_frames,
      gain: GainRamp::silent(),
      buffer: None,
      last_frame_samples: [0.0; MAX_PLAYBACK_CHANNELS],
    };
  }

  /// Replace the active buffer and reset playback to the start.
  ///
  /// # Arguments
  /// - `buffer`: The decoded buffer to schedule.
  ///
  /// # Returns
  /// `()` after updating the active buffer.
  fn set_buffer(&mut self, buffer: Arc<SoundBuffer>) {
    self.buffer = Some(buffer);
    self.cursor_samples = 0;
    return;
  }

  /// Enable or disable looping.
  ///
  /// # Arguments
  /// - `looping`: Whether playback should loop on completion.
  ///
  /// # Returns
  /// `()` after updating the looping flag.
  fn set_looping(&mut self, looping: bool) {
    self.looping = looping;
    return;
  }

  /// Begin playback, or resume if paused.
  ///
  /// # Returns
  /// `()` after updating the transport state.
  fn play(&mut self) {
    self.state = PlaybackState::Playing;
    self.gain.start(1.0, self.ramp_frames);
    return;
  }

  /// Pause playback, preserving the current cursor.
  ///
  /// # Returns
  /// `()` after updating the transport state.
  fn pause(&mut self) {
    self.state = PlaybackState::Paused;
    self.gain.start(0.0, self.ramp_frames);
    return;
  }

  /// Stop playback and reset the cursor to the start.
  ///
  /// # Returns
  /// `()` after updating the transport state.
  fn stop(&mut self) {
    self.state = PlaybackState::Stopped;
    self.cursor_samples = 0;
    self.gain.start(0.0, self.ramp_frames);
    return;
  }

  /// Return the current transport state.
  ///
  /// # Returns
  /// The current `PlaybackState`.
  fn state(&self) -> PlaybackState {
    return self.state;
  }

  /// Return the current interleaved cursor position in samples.
  ///
  /// # Returns
  /// The cursor position as an interleaved sample index.
  fn cursor_samples(&self) -> usize {
    return self.cursor_samples;
  }

  /// Render audio for a callback tick into an output writer.
  ///
  /// # Arguments
  /// - `writer`: Real-time writer for the current callback output buffer.
  ///
  /// # Returns
  /// `()` after writing the output buffer.
  fn render(&mut self, writer: &mut dyn AudioOutputWriter) {
    let writer_channels = writer.channels() as usize;
    let frames = writer.frames();

    if writer_channels == 0 || frames == 0 {
      return;
    }

    if writer_channels > MAX_PLAYBACK_CHANNELS {
      writer.clear();
      return;
    }

    if writer_channels != self.channels {
      writer.clear();
      return;
    }

    if self.state != PlaybackState::Playing && self.gain.is_silent() {
      writer.clear();
      return;
    }

    for frame_index in 0..frames {
      let frame_gain = self.gain.current;

      if self.state == PlaybackState::Playing {
        let Some(buffer) = self.buffer.as_ref() else {
          for channel_index in 0..writer_channels {
            writer.set_sample(frame_index, channel_index, 0.0);
          }
          self.gain.advance_frame();
          continue;
        };

        let samples = buffer.samples();
        let mut frame_start = self.cursor_samples;
        let mut frame_end = frame_start.saturating_add(writer_channels);

        if frame_end > samples.len()
          && self.looping
          && samples.len() >= writer_channels
        {
          self.cursor_samples = 0;
          frame_start = 0;
          frame_end = writer_channels;
        }

        if frame_end <= samples.len() {
          for channel_index in 0..writer_channels {
            let sample = samples
              .get(frame_start.saturating_add(channel_index))
              .copied()
              .unwrap_or(0.0);
            self.last_frame_samples[channel_index] = sample;
            writer.set_sample(frame_index, channel_index, sample * frame_gain);
          }

          self.cursor_samples = frame_end;
          self.gain.advance_frame();
          continue;
        }

        self.state = PlaybackState::Stopped;
        self.cursor_samples = 0;
        self.gain.start(0.0, self.ramp_frames);
      }

      for channel_index in 0..writer_channels {
        let sample = self.last_frame_samples[channel_index];
        writer.set_sample(frame_index, channel_index, sample * frame_gain);
      }

      self.gain.advance_frame();
    }

    return;
  }
}

/// Commands produced by `SoundInstance` transport operations.
#[derive(Debug)]
#[allow(dead_code)]
enum PlaybackCommand {
  StopCurrent,
  SetBuffer {
    instance_id: u64,
    buffer: Arc<SoundBuffer>,
  },
  SetLooping {
    instance_id: u64,
    looping: bool,
  },
  Play {
    instance_id: u64,
  },
  Pause {
    instance_id: u64,
  },
  Stop {
    instance_id: u64,
  },
}

type PlaybackCommandQueue =
  CommandQueue<PlaybackCommand, PLAYBACK_COMMAND_CAPACITY>;

/// Shared, queryable state for the active playback slot.
struct PlaybackSharedState {
  active_instance_id: AtomicU64,
  state: AtomicU8,
}

impl PlaybackSharedState {
  /// Create a new shared playback state initialized to `Stopped`.
  ///
  /// # Returns
  /// A shared state container initialized to instance id `0` and `Stopped`.
  fn new() -> Self {
    return Self {
      active_instance_id: AtomicU64::new(0),
      state: AtomicU8::new(playback_state_to_u8(PlaybackState::Stopped)),
    };
  }

  /// Set the active instance id.
  ///
  /// # Arguments
  /// - `instance_id`: The active instance id.
  ///
  /// # Returns
  /// `()` after updating the active instance id.
  fn set_active_instance_id(&self, instance_id: u64) {
    self
      .active_instance_id
      .store(instance_id, Ordering::Release);
    return;
  }

  /// Return the active instance id.
  ///
  /// # Returns
  /// The active instance id.
  fn active_instance_id(&self) -> u64 {
    return self.active_instance_id.load(Ordering::Acquire);
  }

  /// Set the observable playback state.
  ///
  /// # Arguments
  /// - `state`: The state to store.
  ///
  /// # Returns
  /// `()` after updating the stored state.
  fn set_state(&self, state: PlaybackState) {
    self
      .state
      .store(playback_state_to_u8(state), Ordering::Release);
    return;
  }

  /// Return the observable playback state.
  ///
  /// # Returns
  /// The stored playback state.
  fn state(&self) -> PlaybackState {
    let value = self.state.load(Ordering::Acquire);
    return playback_state_from_u8(value);
  }
}

fn playback_state_to_u8(state: PlaybackState) -> u8 {
  match state {
    PlaybackState::Stopped => {
      return 0;
    }
    PlaybackState::Playing => {
      return 1;
    }
    PlaybackState::Paused => {
      return 2;
    }
  }
}

fn playback_state_from_u8(value: u8) -> PlaybackState {
  match value {
    1 => {
      return PlaybackState::Playing;
    }
    2 => {
      return PlaybackState::Paused;
    }
    _ => {
      return PlaybackState::Stopped;
    }
  }
}

/// A callback-safe controller that drains transport commands and renders audio.
///
/// This type is intended to be owned by the platform audio callback closure.
#[allow(dead_code)]
struct PlaybackController<const COMMAND_CAPACITY: usize> {
  command_queue: Arc<CommandQueue<PlaybackCommand, COMMAND_CAPACITY>>,
  shared_state: Arc<PlaybackSharedState>,
  scheduler: PlaybackScheduler,
}

#[allow(dead_code)]
impl<const COMMAND_CAPACITY: usize> PlaybackController<COMMAND_CAPACITY> {
  /// Create a controller configured for a fixed output channel count.
  ///
  /// # Arguments
  /// - `channels`: Interleaved output channel count.
  /// - `command_queue`: Shared producer/consumer command queue.
  ///
  /// # Returns
  /// A controller initialized to `Stopped` with no active buffer.
  fn new(
    channels: usize,
    command_queue: Arc<CommandQueue<PlaybackCommand, COMMAND_CAPACITY>>,
    shared_state: Arc<PlaybackSharedState>,
  ) -> Self {
    return Self::new_with_ramp_frames(
      channels,
      DEFAULT_GAIN_RAMP_FRAMES,
      command_queue,
      shared_state,
    );
  }

  /// Create a controller with an explicit gain ramp length.
  ///
  /// # Arguments
  /// - `channels`: Interleaved output channel count.
  /// - `ramp_frames`: Gain ramp length in frames.
  /// - `command_queue`: Shared producer/consumer command queue.
  ///
  /// # Returns
  /// A controller initialized to `Stopped` with no active buffer.
  fn new_with_ramp_frames(
    channels: usize,
    ramp_frames: usize,
    command_queue: Arc<CommandQueue<PlaybackCommand, COMMAND_CAPACITY>>,
    shared_state: Arc<PlaybackSharedState>,
  ) -> Self {
    return Self {
      command_queue,
      shared_state,
      scheduler: PlaybackScheduler::new_with_ramp_frames(channels, ramp_frames),
    };
  }

  /// Drain any pending transport commands.
  ///
  /// # Returns
  /// `()` after applying all pending commands.
  fn drain_commands(&mut self) {
    while let Some(command) = self.command_queue.pop() {
      match command {
        PlaybackCommand::StopCurrent => {
          self.scheduler.stop();
          self.shared_state.set_state(PlaybackState::Stopped);
        }
        PlaybackCommand::SetBuffer {
          instance_id,
          buffer,
        } => {
          if instance_id != self.shared_state.active_instance_id() {
            continue;
          }
          self.scheduler.stop();
          self.scheduler.set_looping(false);
          self.scheduler.set_buffer(buffer);
          self.shared_state.set_state(PlaybackState::Stopped);
        }
        PlaybackCommand::SetLooping {
          instance_id,
          looping,
        } => {
          if instance_id != self.shared_state.active_instance_id() {
            continue;
          }
          self.scheduler.set_looping(looping);
        }
        PlaybackCommand::Play { instance_id } => {
          if instance_id != self.shared_state.active_instance_id() {
            continue;
          }
          self.scheduler.play();
          self.shared_state.set_state(PlaybackState::Playing);
        }
        PlaybackCommand::Pause { instance_id } => {
          if instance_id != self.shared_state.active_instance_id() {
            continue;
          }
          self.scheduler.pause();
          self.shared_state.set_state(PlaybackState::Paused);
        }
        PlaybackCommand::Stop { instance_id } => {
          if instance_id != self.shared_state.active_instance_id() {
            continue;
          }
          self.scheduler.stop();
          self.shared_state.set_state(PlaybackState::Stopped);
        }
      }
    }

    return;
  }

  /// Render audio for a callback tick.
  ///
  /// # Arguments
  /// - `writer`: Real-time writer for the current callback output buffer.
  ///
  /// # Returns
  /// `()` after draining commands and writing the output buffer.
  fn render(&mut self, writer: &mut dyn AudioOutputWriter) {
    self.drain_commands();
    self.scheduler.render(writer);
    self.shared_state.set_state(self.scheduler.state());
    return;
  }
}

/// A queryable playback state for a `SoundInstance`.
///
/// This state is observable from the application thread and is intended to
/// provide basic transport visibility.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaybackState {
  /// The sound is currently playing.
  Playing,
  /// The sound is currently paused.
  Paused,
  /// The sound is stopped and positioned at the start.
  Stopped,
}

/// A lightweight handle controlling the active sound playback slot.
///
/// Only the most recently returned `SoundInstance` for an `AudioContext` is
/// considered active. Calls on inactive instances are no-ops and state queries
/// report `Stopped`.
pub struct SoundInstance {
  instance_id: u64,
  command_queue: Arc<PlaybackCommandQueue>,
  shared_state: Arc<PlaybackSharedState>,
}

impl SoundInstance {
  fn is_active(&self) -> bool {
    return self.shared_state.active_instance_id() == self.instance_id;
  }

  /// Begin playback, or resume if paused.
  ///
  /// # Returns
  /// `()` after updating the requested transport state.
  pub fn play(&mut self) {
    if !self.is_active() {
      return;
    }

    let _result = self.command_queue.push(PlaybackCommand::Play {
      instance_id: self.instance_id,
    });
    return;
  }

  /// Pause playback, preserving playback position.
  ///
  /// # Returns
  /// `()` after updating the requested transport state.
  pub fn pause(&mut self) {
    if !self.is_active() {
      return;
    }

    let _result = self.command_queue.push(PlaybackCommand::Pause {
      instance_id: self.instance_id,
    });
    return;
  }

  /// Stop playback and reset position to the start of the buffer.
  ///
  /// # Returns
  /// `()` after updating the requested transport state.
  pub fn stop(&mut self) {
    if !self.is_active() {
      return;
    }

    let _result = self.command_queue.push(PlaybackCommand::Stop {
      instance_id: self.instance_id,
    });
    return;
  }

  /// Enable or disable looping playback.
  ///
  /// # Arguments
  /// - `looping`: Whether the sound should loop on completion.
  ///
  /// # Returns
  /// `()` after updating the looping flag.
  pub fn set_looping(&mut self, looping: bool) {
    if !self.is_active() {
      return;
    }

    let _result = self.command_queue.push(PlaybackCommand::SetLooping {
      instance_id: self.instance_id,
      looping,
    });
    return;
  }

  /// Query the current state of this instance.
  ///
  /// # Returns
  /// The current transport state.
  pub fn state(&self) -> PlaybackState {
    if !self.is_active() {
      return PlaybackState::Stopped;
    }

    return self.shared_state.state();
  }

  /// Convenience query for `state() == PlaybackState::Playing`.
  ///
  /// # Returns
  /// `true` if the instance state is `Playing`.
  pub fn is_playing(&self) -> bool {
    return self.state() == PlaybackState::Playing;
  }

  /// Convenience query for `state() == PlaybackState::Paused`.
  ///
  /// # Returns
  /// `true` if the instance state is `Paused`.
  pub fn is_paused(&self) -> bool {
    return self.state() == PlaybackState::Paused;
  }

  /// Convenience query for `state() == PlaybackState::Stopped`.
  ///
  /// # Returns
  /// `true` if the instance state is `Stopped`.
  pub fn is_stopped(&self) -> bool {
    return self.state() == PlaybackState::Stopped;
  }
}

/// A playback context owning an output device and one active playback slot.
///
/// This type is a placeholder API surface used while sound playback is under
/// active development. It is expected to become fully functional in a
/// subsequent change set.
pub struct AudioContext {
  _output_device: AudioOutputDevice,
  command_queue: Arc<PlaybackCommandQueue>,
  shared_state: Arc<PlaybackSharedState>,
  next_instance_id: u64,
  output_sample_rate: u32,
  output_channels: u16,
}

/// Builder for creating an `AudioContext`.
#[derive(Debug, Clone)]
pub struct AudioContextBuilder {
  sample_rate: Option<u32>,
  channels: Option<u16>,
  label: Option<String>,
}

impl AudioContextBuilder {
  /// Create a builder with engine defaults.
  ///
  /// # Returns
  /// A builder with no explicit configuration requests.
  pub fn new() -> Self {
    return Self {
      sample_rate: None,
      channels: None,
      label: None,
    };
  }

  /// Request an output sample rate.
  ///
  /// # Arguments
  /// - `rate`: Requested output sample rate in frames per second.
  ///
  /// # Returns
  /// The updated builder.
  pub fn with_sample_rate(mut self, rate: u32) -> Self {
    self.sample_rate = Some(rate);
    return self;
  }

  /// Request an output channel count.
  ///
  /// # Arguments
  /// - `channels`: Requested interleaved output channel count.
  ///
  /// # Returns
  /// The updated builder.
  pub fn with_channels(mut self, channels: u16) -> Self {
    self.channels = Some(channels);
    return self;
  }

  /// Attach a label for diagnostics.
  ///
  /// # Arguments
  /// - `label`: A human-readable label used for diagnostics.
  ///
  /// # Returns
  /// The updated builder.
  pub fn with_label(mut self, label: &str) -> Self {
    self.label = Some(label.to_string());
    return self;
  }

  /// Build an `AudioContext` using the requested configuration.
  ///
  /// # Returns
  /// An initialized audio context handle.
  ///
  /// # Errors
  /// Returns an error if the output device cannot be initialized or if the
  /// requested configuration is invalid or unsupported.
  pub fn build(self) -> Result<AudioContext, AudioError> {
    let sample_rate = self.sample_rate.unwrap_or(DEFAULT_OUTPUT_SAMPLE_RATE);
    let channels = self.channels.unwrap_or(DEFAULT_OUTPUT_CHANNELS);

    if channels as usize > MAX_PLAYBACK_CHANNELS {
      return Err(AudioError::InvalidChannels {
        requested: channels,
      });
    }

    let command_queue: Arc<PlaybackCommandQueue> =
      Arc::new(CommandQueue::new());
    let shared_state = Arc::new(PlaybackSharedState::new());

    let command_queue_for_callback = command_queue.clone();
    let shared_state_for_callback = shared_state.clone();

    let mut controller = PlaybackController::new_with_ramp_frames(
      channels as usize,
      DEFAULT_GAIN_RAMP_FRAMES,
      command_queue_for_callback,
      shared_state_for_callback,
    );

    let mut output_builder = AudioOutputDeviceBuilder::new()
      .with_sample_rate(sample_rate)
      .with_channels(channels);

    if let Some(label) = self.label {
      output_builder = output_builder.with_label(&label);
    }

    let output_device = output_builder.build_with_output_callback(
      move |writer, callback_info| {
        if callback_info.sample_rate != sample_rate
          || callback_info.channels != channels
        {
          writer.clear();
          return;
        }

        controller.render(writer);
        return;
      },
    )?;

    return Ok(AudioContext {
      _output_device: output_device,
      command_queue,
      shared_state,
      next_instance_id: 1,
      output_sample_rate: sample_rate,
      output_channels: channels,
    });
  }
}

impl Default for AudioContextBuilder {
  fn default() -> Self {
    return Self::new();
  }
}

impl AudioContext {
  /// Play a decoded `SoundBuffer` through this context.
  ///
  /// # Arguments
  /// - `buffer`: The decoded sound buffer to schedule for playback.
  ///
  /// # Returns
  /// A lightweight `SoundInstance` handle for controlling playback.
  ///
  /// # Errors
  /// Returns [`AudioError::InvalidData`] when the sound buffer does not match
  /// the output configuration, or when the buffer contains no samples. Returns
  /// [`AudioError::Platform`] when the internal callback command queue is full.
  pub fn play_sound(
    &mut self,
    buffer: &SoundBuffer,
  ) -> Result<SoundInstance, AudioError> {
    if buffer.sample_rate() != self.output_sample_rate {
      return Err(AudioError::InvalidData {
        details: format!(
          "sound buffer sample rate did not match output (buffer_sample_rate={}, output_sample_rate={})",
          buffer.sample_rate(),
          self.output_sample_rate
        ),
      });
    }

    if buffer.channels() != self.output_channels {
      return Err(AudioError::InvalidData {
        details: format!(
          "sound buffer channel count did not match output (buffer_channels={}, output_channels={})",
          buffer.channels(),
          self.output_channels
        ),
      });
    }

    if buffer.samples().is_empty() {
      return Err(AudioError::InvalidData {
        details: "sound buffer contained no samples".to_string(),
      });
    }

    let instance_id = self.next_instance_id;
    self.next_instance_id = self.next_instance_id.wrapping_add(1);
    if self.next_instance_id == 0 {
      self.next_instance_id = 1;
    }

    let previous_instance_id = self.shared_state.active_instance_id();
    let previous_state = self.shared_state.state();
    self.shared_state.set_active_instance_id(instance_id);
    self.shared_state.set_state(PlaybackState::Stopped);

    let shared_buffer = Arc::new(buffer.clone());

    let _result = self.command_queue.push(PlaybackCommand::StopCurrent);

    if self
      .command_queue
      .push(PlaybackCommand::SetBuffer {
        instance_id,
        buffer: shared_buffer,
      })
      .is_err()
    {
      self
        .shared_state
        .set_active_instance_id(previous_instance_id);
      self.shared_state.set_state(previous_state);
      return Err(AudioError::Platform {
        details: "audio playback command queue was full (SetBuffer)"
          .to_string(),
      });
    }

    if self
      .command_queue
      .push(PlaybackCommand::Play { instance_id })
      .is_err()
    {
      self
        .shared_state
        .set_active_instance_id(previous_instance_id);
      self.shared_state.set_state(previous_state);
      return Err(AudioError::Platform {
        details: "audio playback command queue was full (Play)".to_string(),
      });
    }

    self.shared_state.set_state(PlaybackState::Playing);

    return Ok(SoundInstance {
      instance_id,
      command_queue: self.command_queue.clone(),
      shared_state: self.shared_state.clone(),
    });
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  /// Command queues MUST preserve FIFO ordering.
  #[test]
  fn command_queue_preserves_order() {
    let queue: CommandQueue<u32, 8> = CommandQueue::new();

    queue.push(1).unwrap();
    queue.push(2).unwrap();
    queue.push(3).unwrap();

    assert_eq!(queue.pop(), Some(1));
    assert_eq!(queue.pop(), Some(2));
    assert_eq!(queue.pop(), Some(3));
    assert_eq!(queue.pop(), None);
    return;
  }

  /// Command queues MUST reject pushes when full.
  #[test]
  fn command_queue_rejects_when_full() {
    let queue: CommandQueue<u32, 2> = CommandQueue::new();

    assert!(queue.push(10).is_ok());
    assert!(queue.push(11).is_ok());
    assert!(matches!(queue.push(12), Err(12)));

    assert_eq!(queue.pop(), Some(10));
    assert_eq!(queue.pop(), Some(11));
    assert_eq!(queue.pop(), None);
    return;
  }

  struct TestAudioOutput {
    channels: u16,
    frames: usize,
    samples: Vec<f32>,
  }

  impl TestAudioOutput {
    fn new(channels: u16, frames: usize) -> Self {
      return Self {
        channels,
        frames,
        samples: vec![0.0; channels as usize * frames],
      };
    }

    fn sample(&self, frame: usize, channel: usize) -> f32 {
      let index = frame
        .saturating_mul(self.channels as usize)
        .saturating_add(channel);
      return self.samples.get(index).copied().unwrap_or(0.0);
    }

    fn max_abs(&self) -> f32 {
      return self.samples.iter().fold(0.0_f32, |accumulator, value| {
        return accumulator.max(value.abs());
      });
    }
  }

  impl AudioOutputWriter for TestAudioOutput {
    fn channels(&self) -> u16 {
      return self.channels;
    }

    fn frames(&self) -> usize {
      return self.frames;
    }

    fn clear(&mut self) {
      for value in self.samples.iter_mut() {
        *value = 0.0;
      }
      return;
    }

    fn set_sample(
      &mut self,
      frame_index: usize,
      channel_index: usize,
      sample: f32,
    ) {
      let index = frame_index
        .saturating_mul(self.channels as usize)
        .saturating_add(channel_index);
      if let Some(value) = self.samples.get_mut(index) {
        *value = sample;
      }
      return;
    }
  }

  fn make_test_buffer(samples: Vec<f32>, channels: u16) -> Arc<SoundBuffer> {
    let buffer =
      SoundBuffer::from_interleaved_samples_for_test(samples, 48_000, channels)
        .expect("test buffer creation failed");
    return Arc::new(buffer);
  }

  /// Scheduler MUST stop and reset the cursor when a buffer completes.
  #[test]
  fn scheduler_stops_after_completion() {
    let buffer = make_test_buffer(vec![0.5, 0.5, 0.5, 0.5], 1);

    let mut scheduler = PlaybackScheduler::new_with_ramp_frames(1, 2);
    scheduler.set_buffer(buffer);
    scheduler.play();

    let mut writer = TestAudioOutput::new(1, 8);
    scheduler.render(&mut writer);

    assert_eq!(scheduler.state(), PlaybackState::Stopped);
    assert_eq!(scheduler.cursor_samples(), 0);

    let mut writer = TestAudioOutput::new(1, 4);
    scheduler.render(&mut writer);
    assert!(writer.max_abs() <= 0.5);

    let mut writer = TestAudioOutput::new(1, 4);
    scheduler.render(&mut writer);
    assert!(writer.max_abs() <= f32::EPSILON);
    return;
  }

  /// Pause MUST preserve cursor and fade to silence.
  #[test]
  fn scheduler_pause_preserves_cursor() {
    let buffer =
      make_test_buffer(vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8], 1);

    let mut scheduler = PlaybackScheduler::new_with_ramp_frames(1, 2);
    scheduler.set_buffer(buffer);
    scheduler.play();

    let mut writer = TestAudioOutput::new(1, 4);
    scheduler.render(&mut writer);

    let cursor_before_pause = scheduler.cursor_samples();
    scheduler.pause();

    let mut writer = TestAudioOutput::new(1, 4);
    scheduler.render(&mut writer);
    assert_eq!(scheduler.cursor_samples(), cursor_before_pause);

    let mut writer = TestAudioOutput::new(1, 4);
    scheduler.render(&mut writer);
    assert!(writer.max_abs() <= f32::EPSILON);
    return;
  }

  /// Stop MUST reset the cursor and fade to silence.
  #[test]
  fn scheduler_stop_resets_cursor() {
    let buffer = make_test_buffer(vec![0.25; 32], 1);

    let mut scheduler = PlaybackScheduler::new_with_ramp_frames(1, 2);
    scheduler.set_buffer(buffer);
    scheduler.play();

    let mut writer = TestAudioOutput::new(1, 8);
    scheduler.render(&mut writer);
    assert!(scheduler.cursor_samples() > 0);

    scheduler.stop();
    assert_eq!(scheduler.state(), PlaybackState::Stopped);
    assert_eq!(scheduler.cursor_samples(), 0);

    let mut writer = TestAudioOutput::new(1, 4);
    scheduler.render(&mut writer);
    let mut writer = TestAudioOutput::new(1, 4);
    scheduler.render(&mut writer);
    assert!(writer.max_abs() <= f32::EPSILON);
    return;
  }

  /// Looping MUST wrap the cursor and continue producing samples.
  #[test]
  fn scheduler_looping_wraps() {
    let buffer = make_test_buffer(vec![0.1, 0.2], 1);

    let mut scheduler = PlaybackScheduler::new_with_ramp_frames(1, 0);
    scheduler.set_buffer(buffer);
    scheduler.set_looping(true);
    scheduler.play();

    let mut writer = TestAudioOutput::new(1, 4);
    scheduler.render(&mut writer);

    assert_eq!(scheduler.state(), PlaybackState::Playing);
    assert!((writer.sample(0, 0) - 0.1).abs() <= 1e-6);
    assert!((writer.sample(1, 0) - 0.2).abs() <= 1e-6);
    assert!((writer.sample(2, 0) - 0.1).abs() <= 1e-6);
    assert!((writer.sample(3, 0) - 0.2).abs() <= 1e-6);
    return;
  }

  /// Transport transitions MUST avoid hard discontinuities.
  #[test]
  fn scheduler_pause_is_continuous_at_transition_boundary() {
    let buffer = make_test_buffer(vec![0.5; 64], 1);

    let mut scheduler = PlaybackScheduler::new_with_ramp_frames(1, 2);
    scheduler.set_buffer(buffer);
    scheduler.play();

    let mut writer = TestAudioOutput::new(1, 8);
    scheduler.render(&mut writer);
    let last = writer.sample(7, 0);

    scheduler.pause();

    let mut writer = TestAudioOutput::new(1, 1);
    scheduler.render(&mut writer);
    let first = writer.sample(0, 0);

    assert!((last - first).abs() <= 1e-6);
    return;
  }

  /// Controllers MUST drain queued commands before rendering audio.
  #[test]
  fn controller_drains_commands_before_render() {
    let command_queue: Arc<CommandQueue<PlaybackCommand, 16>> =
      Arc::new(CommandQueue::new());
    let shared_state = Arc::new(PlaybackSharedState::new());
    let buffer = make_test_buffer(vec![0.1, 0.2], 1);

    shared_state.set_active_instance_id(1);

    command_queue
      .push(PlaybackCommand::SetBuffer {
        instance_id: 1,
        buffer,
      })
      .unwrap();
    command_queue
      .push(PlaybackCommand::SetLooping {
        instance_id: 1,
        looping: true,
      })
      .unwrap();
    command_queue
      .push(PlaybackCommand::Play { instance_id: 1 })
      .unwrap();

    let mut controller = PlaybackController::new_with_ramp_frames(
      1,
      0,
      command_queue,
      shared_state.clone(),
    );

    let mut writer = TestAudioOutput::new(1, 4);
    controller.render(&mut writer);

    assert!((writer.sample(0, 0) - 0.1).abs() <= 1e-6);
    assert!((writer.sample(1, 0) - 0.2).abs() <= 1e-6);
    assert!((writer.sample(2, 0) - 0.1).abs() <= 1e-6);
    assert!((writer.sample(3, 0) - 0.2).abs() <= 1e-6);
    assert_eq!(shared_state.state(), PlaybackState::Playing);
    return;
  }

  /// Controllers MUST ignore transport commands for inactive instances.
  #[test]
  fn controller_ignores_commands_for_inactive_instance() {
    let command_queue: Arc<CommandQueue<PlaybackCommand, 16>> =
      Arc::new(CommandQueue::new());
    let shared_state = Arc::new(PlaybackSharedState::new());
    let buffer = make_test_buffer(vec![0.1, 0.2], 1);

    shared_state.set_active_instance_id(2);

    let mut controller = PlaybackController::new_with_ramp_frames(
      1,
      0,
      command_queue.clone(),
      shared_state,
    );

    command_queue
      .push(PlaybackCommand::SetBuffer {
        instance_id: 1,
        buffer,
      })
      .unwrap();
    command_queue
      .push(PlaybackCommand::Play { instance_id: 1 })
      .unwrap();

    let mut writer = TestAudioOutput::new(1, 1);
    controller.render(&mut writer);
    assert!(writer.max_abs() <= f32::EPSILON);
    return;
  }
}
