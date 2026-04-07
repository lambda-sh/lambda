use std::{
  cell::RefCell,
  collections::HashMap,
  sync::{
    Arc,
    Mutex,
    Weak,
  },
};

use super::cpu::CpuEvent;

thread_local! {
  static THREAD_LOCAL_EVENT_BUFFERS: RefCell<HashMap<usize, Arc<Mutex<ThreadLocalEventBuffer>>>> =
    RefCell::new(HashMap::new());
}

/// Stores per-thread CPU events for a single collector.
#[derive(Debug)]
pub(crate) struct ThreadLocalEventBuffer {
  events: Vec<CpuEvent>,
  max_capacity_observed: usize,
}

impl ThreadLocalEventBuffer {
  /// Creates a fixed-capacity thread-local event buffer.
  ///
  /// # Arguments
  /// - `capacity`: Maximum number of events held before rollover.
  ///
  /// # Returns
  /// Returns a newly allocated thread-local event buffer.
  pub(crate) fn new(capacity: usize) -> Self {
    return Self {
      events: Vec::with_capacity(capacity),
      max_capacity_observed: capacity,
    };
  }

  /// Returns whether the thread-local buffer is currently empty.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns `true` when no buffered events are present.
  pub(crate) fn is_empty(&self) -> bool {
    return self.events.is_empty();
  }

  /// Returns whether the thread-local buffer has reached capacity.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns `true` when the next pushed event would require a rollover.
  pub(crate) fn is_full(&self) -> bool {
    return self.events.len() == self.events.capacity();
  }

  /// Pushes a CPU event into the thread-local buffer.
  ///
  /// # Arguments
  /// - `event`: CPU event to append to the buffer.
  ///
  /// # Returns
  /// This method does not return a value.
  pub(crate) fn push(&mut self, event: CpuEvent) {
    self.events.push(event);
    self.max_capacity_observed =
      self.max_capacity_observed.max(self.events.capacity());
  }

  /// Moves all buffered events into a shared destination vector.
  ///
  /// # Arguments
  /// - `destination`: Shared destination that receives the drained events.
  ///
  /// # Returns
  /// This method does not return a value.
  pub(crate) fn drain_into(&mut self, destination: &mut Vec<CpuEvent>) {
    destination.append(&mut self.events);
  }

  /// Returns the configured buffer capacity.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the current fixed capacity of the event buffer.
  #[cfg(test)]
  pub(crate) fn capacity(&self) -> usize {
    return self.events.capacity();
  }

  /// Returns the largest observed backing capacity for the buffer.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the maximum vector capacity observed while recording events.
  #[cfg(test)]
  pub(crate) fn max_capacity_observed(&self) -> usize {
    return self.max_capacity_observed;
  }
}

/// Returns the thread-local buffer for a collector, creating it on first use.
///
/// # Arguments
/// - `collector_id`: Stable identifier for the owning CPU collector.
/// - `capacity`: Maximum number of buffered events before rollover.
/// - `register`: Callback used to register the created buffer with the
///   collector so non-active threads can be flushed later.
///
/// # Returns
/// Returns a shared handle to the per-thread buffer for the collector.
pub(crate) fn get_or_insert_thread_local_buffer<F>(
  collector_id: usize,
  capacity: usize,
  register: F,
) -> Arc<Mutex<ThreadLocalEventBuffer>>
where
  F: FnOnce(Weak<Mutex<ThreadLocalEventBuffer>>),
{
  return THREAD_LOCAL_EVENT_BUFFERS.with(|buffers| {
    let mut buffers = buffers.borrow_mut();

    if let Some(buffer) = buffers.get(&collector_id) {
      return Arc::clone(buffer);
    }

    let buffer = Arc::new(Mutex::new(ThreadLocalEventBuffer::new(capacity)));
    register(Arc::downgrade(&buffer));
    buffers.insert(collector_id, Arc::clone(&buffer));

    return buffer;
  });
}
