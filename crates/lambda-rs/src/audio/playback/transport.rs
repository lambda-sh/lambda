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

use super::{
  PlaybackState,
  PLAYBACK_COMMAND_CAPACITY,
};
use crate::audio::SoundBuffer;

/// A fixed-capacity, single-producer/single-consumer queue.
///
/// The queue is designed for real-time audio callbacks:
/// - `push` and `pop` MUST NOT block.
/// - `pop` MUST NOT allocate.
///
/// # Safety
/// This type is only sound when used as SPSC (exactly one producer thread and
/// one consumer thread).
pub(super) struct CommandQueue<T, const CAPACITY: usize> {
  buffer: [UnsafeCell<MaybeUninit<T>>; CAPACITY],
  head: AtomicUsize,
  tail: AtomicUsize,
}

unsafe impl<T: Send, const CAPACITY: usize> Send for CommandQueue<T, CAPACITY> {}
unsafe impl<T: Send, const CAPACITY: usize> Sync for CommandQueue<T, CAPACITY> {}

impl<T, const CAPACITY: usize> CommandQueue<T, CAPACITY> {
  /// Create a new empty queue.
  ///
  /// # Returns
  /// A queue with a fixed capacity.
  pub(super) fn new() -> Self {
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
  pub(super) fn push(&self, value: T) -> Result<(), T> {
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
  pub(super) fn pop(&self) -> Option<T> {
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

/// Commands produced by `SoundInstance` transport operations.
#[derive(Debug)]
pub(super) enum PlaybackCommand {
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

pub(super) type PlaybackCommandQueue =
  CommandQueue<PlaybackCommand, PLAYBACK_COMMAND_CAPACITY>;

/// Shared, queryable state for the active playback slot.
pub(super) struct PlaybackSharedState {
  active_instance_id: AtomicU64,
  state: AtomicU8,
}

impl PlaybackSharedState {
  /// Create a new shared playback state initialized to `Stopped`.
  ///
  /// # Returns
  /// A shared state container initialized to instance id `0` and `Stopped`.
  pub(super) fn new() -> Self {
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
  pub(super) fn set_active_instance_id(&self, instance_id: u64) {
    self
      .active_instance_id
      .store(instance_id, Ordering::Release);
    return;
  }

  /// Return the active instance id.
  ///
  /// # Returns
  /// The active instance id.
  pub(super) fn active_instance_id(&self) -> u64 {
    return self.active_instance_id.load(Ordering::Acquire);
  }

  /// Set the observable playback state.
  ///
  /// # Arguments
  /// - `state`: The state to store.
  ///
  /// # Returns
  /// `()` after updating the stored playback state.
  pub(super) fn set_state(&self, state: PlaybackState) {
    self
      .state
      .store(playback_state_to_u8(state), Ordering::Release);
    return;
  }

  /// Return the observable playback state.
  ///
  /// # Returns
  /// The stored playback state.
  pub(super) fn state(&self) -> PlaybackState {
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
    assert!(queue.pop().is_none());
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
    assert!(queue.pop().is_none());
    return;
  }
}
