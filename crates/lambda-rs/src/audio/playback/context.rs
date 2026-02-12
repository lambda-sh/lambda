use std::sync::Arc;

use super::{
  CommandQueue,
  PlaybackCommand,
  PlaybackCommandQueue,
  PlaybackController,
  PlaybackSharedState,
  PlaybackState,
  DEFAULT_GAIN_RAMP_FRAMES,
  DEFAULT_OUTPUT_CHANNELS,
  DEFAULT_OUTPUT_SAMPLE_RATE,
  MAX_PLAYBACK_CHANNELS,
};
use crate::audio::{
  AudioError,
  AudioOutputDevice,
  AudioOutputDeviceBuilder,
  SoundBuffer,
};

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
pub struct AudioContext {
  _output_device: Option<AudioOutputDevice>,
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
      _output_device: Some(output_device),
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

  fn create_test_context(sample_rate: u32, channels: u16) -> AudioContext {
    return AudioContext {
      _output_device: None,
      command_queue: Arc::new(CommandQueue::new()),
      shared_state: Arc::new(PlaybackSharedState::new()),
      next_instance_id: 1,
      output_sample_rate: sample_rate,
      output_channels: channels,
    };
  }

  fn create_test_sound_buffer(
    sample_rate: u32,
    channels: u16,
    frames: usize,
  ) -> SoundBuffer {
    let sample_count = frames * channels as usize;
    let samples = vec![0.0; sample_count];
    return SoundBuffer::from_interleaved_samples_for_test(
      samples,
      sample_rate,
      channels,
    )
    .expect("test sound buffer must be valid");
  }

  fn fill_command_queue(queue: &PlaybackCommandQueue) {
    while queue.push(PlaybackCommand::StopCurrent).is_ok() {}
    return;
  }

  /// `SoundInstance` methods MUST be no-ops when the instance is inactive.
  #[test]
  fn sound_instance_is_no_op_when_inactive() {
    let command_queue: Arc<PlaybackCommandQueue> =
      Arc::new(CommandQueue::new());
    let shared_state = Arc::new(PlaybackSharedState::new());

    shared_state.set_active_instance_id(1);
    shared_state.set_state(PlaybackState::Playing);

    let mut instance = SoundInstance {
      instance_id: 1,
      command_queue: command_queue.clone(),
      shared_state: shared_state.clone(),
    };

    assert_eq!(instance.state(), PlaybackState::Playing);
    assert!(instance.is_playing());
    assert!(!instance.is_paused());
    assert!(!instance.is_stopped());

    shared_state.set_active_instance_id(2);
    shared_state.set_state(PlaybackState::Paused);

    assert_eq!(instance.state(), PlaybackState::Stopped);
    assert!(!instance.is_playing());
    assert!(!instance.is_paused());
    assert!(instance.is_stopped());

    instance.play();
    instance.pause();
    instance.stop();
    instance.set_looping(true);

    assert!(command_queue.pop().is_none());
    return;
  }

  /// `SoundInstance` MUST enqueue commands when it is the active instance.
  #[test]
  fn sound_instance_enqueues_commands_when_active() {
    let command_queue: Arc<PlaybackCommandQueue> =
      Arc::new(CommandQueue::new());
    let shared_state = Arc::new(PlaybackSharedState::new());

    shared_state.set_active_instance_id(7);
    shared_state.set_state(PlaybackState::Stopped);

    let mut instance = SoundInstance {
      instance_id: 7,
      command_queue: command_queue.clone(),
      shared_state: shared_state.clone(),
    };

    instance.play();
    instance.pause();
    instance.set_looping(true);
    instance.stop();

    assert!(matches!(
      command_queue.pop(),
      Some(PlaybackCommand::Play { instance_id: 7 })
    ));
    assert!(matches!(
      command_queue.pop(),
      Some(PlaybackCommand::Pause { instance_id: 7 })
    ));
    assert!(matches!(
      command_queue.pop(),
      Some(PlaybackCommand::SetLooping {
        instance_id: 7,
        looping: true
      })
    ));
    assert!(matches!(
      command_queue.pop(),
      Some(PlaybackCommand::Stop { instance_id: 7 })
    ));
    assert!(command_queue.pop().is_none());
    return;
  }

  /// `SoundInstance` state queries MUST reflect the shared state when active.
  #[test]
  fn sound_instance_state_follows_shared_state_when_active() {
    let command_queue: Arc<PlaybackCommandQueue> =
      Arc::new(CommandQueue::new());
    let shared_state = Arc::new(PlaybackSharedState::new());

    shared_state.set_active_instance_id(1);

    let instance = SoundInstance {
      instance_id: 1,
      command_queue,
      shared_state: shared_state.clone(),
    };

    shared_state.set_state(PlaybackState::Stopped);
    assert_eq!(instance.state(), PlaybackState::Stopped);
    assert!(instance.is_stopped());
    assert!(!instance.is_playing());
    assert!(!instance.is_paused());

    shared_state.set_state(PlaybackState::Playing);
    assert_eq!(instance.state(), PlaybackState::Playing);
    assert!(instance.is_playing());
    assert!(!instance.is_paused());
    assert!(!instance.is_stopped());

    shared_state.set_state(PlaybackState::Paused);
    assert_eq!(instance.state(), PlaybackState::Paused);
    assert!(!instance.is_playing());
    assert!(instance.is_paused());
    assert!(!instance.is_stopped());
    return;
  }

  /// The builder MUST reject unsupported channel counts before device init.
  #[test]
  fn audio_context_builder_rejects_too_many_channels() {
    let result = AudioContextBuilder::new()
      .with_channels((MAX_PLAYBACK_CHANNELS + 1) as u16)
      .build();

    assert!(matches!(
      result,
      Err(AudioError::InvalidChannels { requested })
        if requested == (MAX_PLAYBACK_CHANNELS + 1) as u16
    ));
    return;
  }

  /// Builder configuration MUST store requested fields.
  #[test]
  fn audio_context_builder_stores_configuration() {
    let builder = AudioContextBuilder::new()
      .with_sample_rate(48_000)
      .with_channels(2)
      .with_label("test-context");

    assert_eq!(builder.sample_rate, Some(48_000));
    assert_eq!(builder.channels, Some(2));
    assert_eq!(builder.label.as_deref(), Some("test-context"));
    return;
  }

  /// The builder MUST reject invalid sample rates before device selection.
  #[test]
  fn audio_context_builder_rejects_invalid_sample_rate() {
    let result = AudioContextBuilder::new().with_sample_rate(0).build();
    assert!(matches!(
      result,
      Err(AudioError::InvalidSampleRate { requested: 0 })
    ));
    return;
  }

  /// The builder MUST reject invalid channel counts before device selection.
  #[test]
  fn audio_context_builder_rejects_invalid_channels() {
    let result = AudioContextBuilder::new().with_channels(0).build();
    assert!(matches!(
      result,
      Err(AudioError::InvalidChannels { requested: 0 })
    ));
    return;
  }

  /// `play_sound` MUST reject sound buffers with mismatched sample rates.
  #[test]
  fn play_sound_rejects_sample_rate_mismatch() {
    let mut context = create_test_context(48_000, 2);
    let buffer = create_test_sound_buffer(44_100, 2, 4);

    let result = context.play_sound(&buffer);
    assert!(matches!(result, Err(AudioError::InvalidData { .. })));

    assert!(context.command_queue.pop().is_none());
    assert_eq!(context.shared_state.active_instance_id(), 0);
    assert_eq!(context.shared_state.state(), PlaybackState::Stopped);
    return;
  }

  /// `play_sound` MUST reject sound buffers with mismatched channel counts.
  #[test]
  fn play_sound_rejects_channel_mismatch() {
    let mut context = create_test_context(48_000, 2);
    let buffer = create_test_sound_buffer(48_000, 1, 4);

    let result = context.play_sound(&buffer);
    assert!(matches!(result, Err(AudioError::InvalidData { .. })));

    assert!(context.command_queue.pop().is_none());
    assert_eq!(context.shared_state.active_instance_id(), 0);
    assert_eq!(context.shared_state.state(), PlaybackState::Stopped);
    return;
  }

  /// `play_sound` MUST reject empty sound buffers.
  #[test]
  fn play_sound_rejects_empty_samples() {
    let mut context = create_test_context(48_000, 2);
    let buffer = create_test_sound_buffer(48_000, 2, 0 /* frames */);

    let result = context.play_sound(&buffer);
    assert!(matches!(result, Err(AudioError::InvalidData { .. })));

    assert!(context.command_queue.pop().is_none());
    assert_eq!(context.shared_state.active_instance_id(), 0);
    assert_eq!(context.shared_state.state(), PlaybackState::Stopped);
    return;
  }

  /// `play_sound` MUST schedule stop, buffer, then play commands.
  #[test]
  fn play_sound_enqueues_commands_and_updates_state() {
    let mut context = create_test_context(48_000, 2);
    let buffer = create_test_sound_buffer(48_000, 2, 4);

    let instance = context.play_sound(&buffer).expect("must play sound");
    assert_eq!(instance.instance_id, 1);
    assert_eq!(context.shared_state.active_instance_id(), 1);
    assert_eq!(context.shared_state.state(), PlaybackState::Playing);
    assert_eq!(instance.state(), PlaybackState::Playing);

    assert!(matches!(
      context.command_queue.pop(),
      Some(PlaybackCommand::StopCurrent)
    ));
    match context.command_queue.pop() {
      Some(PlaybackCommand::SetBuffer {
        instance_id,
        buffer: scheduled_buffer,
      }) => {
        assert_eq!(instance_id, 1);
        assert_eq!(scheduled_buffer.as_ref(), &buffer);
      }
      other => {
        panic!("expected SetBuffer command, got {other:?}");
      }
    }
    assert!(matches!(
      context.command_queue.pop(),
      Some(PlaybackCommand::Play { instance_id: 1 })
    ));
    assert!(context.command_queue.pop().is_none());
    return;
  }

  /// `play_sound` MUST restore previous state when the queue is full.
  #[test]
  fn play_sound_restores_state_when_queue_full_for_set_buffer() {
    let mut context = create_test_context(48_000, 2);
    let buffer = create_test_sound_buffer(48_000, 2, 4);

    context.shared_state.set_active_instance_id(9);
    context.shared_state.set_state(PlaybackState::Paused);

    fill_command_queue(&context.command_queue);

    let result = context.play_sound(&buffer);
    assert!(matches!(result, Err(AudioError::Platform { .. })));

    assert_eq!(context.shared_state.active_instance_id(), 9);
    assert_eq!(context.shared_state.state(), PlaybackState::Paused);
    return;
  }

  /// `play_sound` MUST restore previous state when play cannot be enqueued.
  #[test]
  fn play_sound_restores_state_when_queue_full_for_play() {
    let mut context = create_test_context(48_000, 2);
    let buffer = create_test_sound_buffer(48_000, 2, 4);

    context.shared_state.set_active_instance_id(3);
    context.shared_state.set_state(PlaybackState::Paused);

    fill_command_queue(&context.command_queue);
    let _first_popped = context.command_queue.pop();
    let _second_popped = context.command_queue.pop();

    let result = context.play_sound(&buffer);
    assert!(matches!(result, Err(AudioError::Platform { .. })));

    assert_eq!(context.shared_state.active_instance_id(), 3);
    assert_eq!(context.shared_state.state(), PlaybackState::Paused);
    return;
  }

  /// Instance ids MUST wrap without using id `0`.
  #[test]
  fn play_sound_instance_id_wraps_to_one() {
    let mut context = create_test_context(48_000, 2);
    context.next_instance_id = u64::MAX;

    let buffer = create_test_sound_buffer(48_000, 2, 4);

    let instance = context.play_sound(&buffer).expect("must play sound");
    assert_eq!(instance.instance_id, u64::MAX);
    assert_eq!(context.next_instance_id, 1);
    return;
  }
}
