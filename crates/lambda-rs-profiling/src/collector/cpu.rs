use std::{
  sync::{
    atomic::{
      AtomicUsize,
      Ordering,
    },
    Arc,
    Mutex,
    Weak,
  },
  thread,
  thread::ThreadId,
  time::Instant,
};

use super::thread_local::{
  get_or_insert_thread_local_buffer,
  ThreadLocalEventBuffer,
};

static NEXT_CPU_COLLECTOR_ID: AtomicUsize = AtomicUsize::new(1);

/// Default fixed-capacity size for per-thread CPU event buffers.
pub(crate) const DEFAULT_THREAD_LOCAL_BUFFER_CAPACITY: usize = 256;

/// Identifies the kind of CPU profiling event that was recorded.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CpuEventKind {
  SpanBegin,
  SpanEnd,
  Counter { value: u64 },
  Instant,
}

/// Stores one CPU profiling event in collector-relative time.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CpuEvent {
  pub(crate) name: &'static str,
  pub(crate) kind: CpuEventKind,
  pub(crate) timestamp_ns: u64,
  pub(crate) thread_id: ThreadId,
}

impl CpuEvent {
  /// Creates a span-begin event.
  ///
  /// # Arguments
  /// - `name`: Stable name for the span.
  /// - `timestamp_ns`: Collector-relative timestamp for the event.
  ///
  /// # Returns
  /// Returns a CPU event that marks a span begin point.
  pub(crate) fn span_begin(name: &'static str, timestamp_ns: u64) -> Self {
    return Self {
      name,
      kind: CpuEventKind::SpanBegin,
      timestamp_ns,
      thread_id: thread::current().id(),
    };
  }

  /// Creates a span-end event.
  ///
  /// # Arguments
  /// - `name`: Stable name for the span.
  /// - `timestamp_ns`: Collector-relative timestamp for the event.
  ///
  /// # Returns
  /// Returns a CPU event that marks a span end point.
  pub(crate) fn span_end(name: &'static str, timestamp_ns: u64) -> Self {
    return Self {
      name,
      kind: CpuEventKind::SpanEnd,
      timestamp_ns,
      thread_id: thread::current().id(),
    };
  }

  /// Creates a counter event.
  ///
  /// # Arguments
  /// - `name`: Stable counter name.
  /// - `timestamp_ns`: Collector-relative timestamp for the event.
  /// - `value`: Recorded counter value.
  ///
  /// # Returns
  /// Returns a CPU event that stores the provided counter value.
  pub(crate) fn counter(
    name: &'static str,
    timestamp_ns: u64,
    value: u64,
  ) -> Self {
    return Self {
      name,
      kind: CpuEventKind::Counter { value },
      timestamp_ns,
      thread_id: thread::current().id(),
    };
  }

  /// Creates an instant event.
  ///
  /// # Arguments
  /// - `name`: Stable instant name.
  /// - `timestamp_ns`: Collector-relative timestamp for the event.
  ///
  /// # Returns
  /// Returns a CPU event that marks an instantaneous point in time.
  pub(crate) fn instant(name: &'static str, timestamp_ns: u64) -> Self {
    return Self {
      name,
      kind: CpuEventKind::Instant,
      timestamp_ns,
      thread_id: thread::current().id(),
    };
  }
}

#[derive(Debug)]
struct CpuCollectorState {
  events: Vec<CpuEvent>,
  buffer_rollovers: usize,
}

impl CpuCollectorState {
  /// Creates empty collector state.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns a collector state with no aggregated events.
  fn new() -> Self {
    return Self {
      events: Vec::new(),
      buffer_rollovers: 0,
    };
  }
}

/// Collects CPU profiling events from thread-local buffers.
#[derive(Debug)]
pub(crate) struct CpuCollector {
  collector_id: usize,
  time_origin: Instant,
  thread_local_buffer_capacity: usize,
  thread_local_buffers: Mutex<Vec<Weak<Mutex<ThreadLocalEventBuffer>>>>,
  state: Mutex<CpuCollectorState>,
}

impl CpuCollector {
  /// Creates a CPU collector with the default per-thread buffer capacity.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns a shared CPU collector ready to receive events.
  pub(crate) fn new() -> Arc<Self> {
    return Self::with_thread_local_buffer_capacity(
      DEFAULT_THREAD_LOCAL_BUFFER_CAPACITY,
    );
  }

  /// Creates a CPU collector with an explicit per-thread buffer capacity.
  ///
  /// # Arguments
  /// - `capacity`: Maximum events each thread-local buffer can hold before a
  ///   rollover occurs.
  ///
  /// # Returns
  /// Returns a shared CPU collector with the requested buffer capacity.
  pub(crate) fn with_thread_local_buffer_capacity(
    capacity: usize,
  ) -> Arc<Self> {
    return Arc::new(Self {
      collector_id: NEXT_CPU_COLLECTOR_ID.fetch_add(1, Ordering::Relaxed),
      time_origin: Instant::now(),
      thread_local_buffer_capacity: capacity.max(1),
      thread_local_buffers: Mutex::new(Vec::new()),
      state: Mutex::new(CpuCollectorState::new()),
    });
  }

  /// Records the beginning of a named CPU span.
  ///
  /// # Arguments
  /// - `name`: Stable span name.
  /// - `started_at`: Instant captured at span start.
  ///
  /// # Returns
  /// This method does not return a value.
  pub(crate) fn record_span_begin(
    self: &Arc<Self>,
    name: &'static str,
    started_at: Instant,
  ) {
    self
      .record_event(CpuEvent::span_begin(name, self.timestamp_ns(started_at)));
  }

  /// Records the end of a named CPU span.
  ///
  /// # Arguments
  /// - `name`: Stable span name.
  /// - `ended_at`: Instant captured at span end.
  ///
  /// # Returns
  /// This method does not return a value.
  pub(crate) fn record_span_end(
    self: &Arc<Self>,
    name: &'static str,
    ended_at: Instant,
  ) {
    self.record_event(CpuEvent::span_end(name, self.timestamp_ns(ended_at)));
  }

  /// Records a named CPU counter event.
  ///
  /// # Arguments
  /// - `name`: Stable counter name.
  /// - `value`: Counter value recorded by the caller.
  ///
  /// # Returns
  /// This method does not return a value.
  pub(crate) fn record_counter(
    self: &Arc<Self>,
    name: &'static str,
    value: u64,
  ) {
    self.record_event(CpuEvent::counter(name, self.now_ns(), value));
  }

  /// Records a named CPU instant event.
  ///
  /// # Arguments
  /// - `name`: Stable instant name.
  ///
  /// # Returns
  /// This method does not return a value.
  pub(crate) fn record_instant(self: &Arc<Self>, name: &'static str) {
    self.record_event(CpuEvent::instant(name, self.now_ns()));
  }

  /// Flushes all registered thread-local buffers into shared collector state.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// This method does not return a value.
  pub(crate) fn flush_thread_local_buffers(&self) {
    let mut thread_local_buffers = self
      .thread_local_buffers
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());

    thread_local_buffers.retain(|buffer| {
      let Some(buffer) = buffer.upgrade() else {
        return false;
      };

      let mut buffer = buffer
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
      self.drain_buffer(&mut buffer, false);

      return true;
    });
  }

  /// Converts an [`Instant`] into a collector-relative nanosecond timestamp.
  ///
  /// # Arguments
  /// - `instant`: Instant to convert into collector-relative time.
  ///
  /// # Returns
  /// Returns the elapsed nanoseconds since the collector was created.
  fn timestamp_ns(&self, instant: Instant) -> u64 {
    let duration = instant
      .checked_duration_since(self.time_origin)
      .unwrap_or_default();

    return duration.as_nanos() as u64;
  }

  /// Returns the current collector-relative timestamp.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the elapsed nanoseconds since the collector was created.
  fn now_ns(&self) -> u64 {
    return self.time_origin.elapsed().as_nanos() as u64;
  }

  /// Registers a thread-local buffer so non-active threads can be flushed.
  ///
  /// # Arguments
  /// - `buffer`: Weak handle to the thread-local buffer owned by a thread.
  ///
  /// # Returns
  /// This method does not return a value.
  fn register_thread_local_buffer(
    &self,
    buffer: Weak<Mutex<ThreadLocalEventBuffer>>,
  ) {
    let mut thread_local_buffers = self
      .thread_local_buffers
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    thread_local_buffers.push(buffer);
  }

  /// Records an event into the current thread-local buffer.
  ///
  /// # Arguments
  /// - `event`: CPU event to record.
  ///
  /// # Returns
  /// This method does not return a value.
  fn record_event(self: &Arc<Self>, event: CpuEvent) {
    let thread_local_buffer = get_or_insert_thread_local_buffer(
      self.collector_id,
      self.thread_local_buffer_capacity,
      |buffer| self.register_thread_local_buffer(buffer),
    );
    let mut thread_local_buffer = thread_local_buffer
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());

    if thread_local_buffer.is_full() {
      self.drain_buffer(&mut thread_local_buffer, true);
    }

    thread_local_buffer.push(event);
  }

  /// Drains a thread-local buffer into shared collector state.
  ///
  /// # Arguments
  /// - `buffer`: Buffer to drain.
  /// - `is_rollover`: Whether this drain was triggered by buffer rollover.
  ///
  /// # Returns
  /// This method does not return a value.
  fn drain_buffer(
    &self,
    buffer: &mut ThreadLocalEventBuffer,
    is_rollover: bool,
  ) {
    if buffer.is_empty() {
      return;
    }

    let mut state = self
      .state
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());

    if is_rollover {
      state.buffer_rollovers += 1;
    }

    buffer.drain_into(&mut state.events);
  }

  /// Returns a snapshot of the collected CPU events for test assertions.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns a snapshot containing the collected events and rollover stats.
  #[cfg(test)]
  pub(crate) fn snapshot(&self) -> CpuCollectorSnapshot {
    self.flush_thread_local_buffers();

    let state = self
      .state
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());

    return CpuCollectorSnapshot {
      events: state.events.clone(),
      buffer_rollovers: state.buffer_rollovers,
      thread_local_buffer_capacity: self.thread_local_buffer_capacity,
      max_thread_local_buffer_capacity_observed: self
        .max_thread_local_buffer_capacity_observed(),
    };
  }

  /// Returns the maximum observed capacity across registered thread-local
  /// buffers.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the largest backing capacity observed by any registered
  /// thread-local buffer.
  #[cfg(test)]
  fn max_thread_local_buffer_capacity_observed(&self) -> usize {
    let thread_local_buffers = self
      .thread_local_buffers
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut max_capacity_observed = 0;

    for buffer in thread_local_buffers.iter() {
      let Some(buffer) = buffer.upgrade() else {
        continue;
      };

      let buffer = buffer
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
      max_capacity_observed = max_capacity_observed
        .max(buffer.max_capacity_observed())
        .max(buffer.capacity());
    }

    return max_capacity_observed;
  }
}

/// Captures a stable CPU collector snapshot for unit tests.
#[cfg(test)]
#[derive(Debug)]
pub(crate) struct CpuCollectorSnapshot {
  pub(crate) events: Vec<CpuEvent>,
  pub(crate) buffer_rollovers: usize,
  pub(crate) thread_local_buffer_capacity: usize,
  pub(crate) max_thread_local_buffer_capacity_observed: usize,
}

#[cfg(test)]
mod cpu_tests {
  use std::{
    sync::Arc,
    time::Instant,
  };

  use super::{
    CpuCollector,
    CpuEventKind,
  };
  use crate::profiler::ScopeGuard;

  #[test]
  fn cpu_scope_guard_closes_exactly_once() {
    let collector = CpuCollector::with_thread_local_buffer_capacity(8);
    let started_at = Instant::now();

    collector.record_span_begin("tests.cpu.scope_once", started_at);

    let mut guard = ScopeGuard::enabled(
      "tests.cpu.scope_once",
      Arc::clone(&collector),
      started_at,
    );

    guard.close();
    drop(guard);

    let snapshot = collector.snapshot();

    assert_eq!(snapshot.events.len(), 2);
    assert_eq!(snapshot.events[0].kind, CpuEventKind::SpanBegin);
    assert_eq!(snapshot.events[1].kind, CpuEventKind::SpanEnd);
  }

  #[test]
  fn cpu_nested_scopes_preserve_begin_end_order() {
    let collector = CpuCollector::with_thread_local_buffer_capacity(8);
    let outer_started_at = Instant::now();

    collector.record_span_begin("tests.cpu.outer", outer_started_at);
    let outer_guard = ScopeGuard::enabled(
      "tests.cpu.outer",
      Arc::clone(&collector),
      outer_started_at,
    );

    let inner_started_at = Instant::now();
    collector.record_span_begin("tests.cpu.inner", inner_started_at);
    let inner_guard = ScopeGuard::enabled(
      "tests.cpu.inner",
      Arc::clone(&collector),
      inner_started_at,
    );

    drop(inner_guard);
    drop(outer_guard);

    let snapshot = collector.snapshot();

    assert_eq!(snapshot.events.len(), 4);
    assert_eq!(snapshot.events[0].name, "tests.cpu.outer");
    assert_eq!(snapshot.events[0].kind, CpuEventKind::SpanBegin);
    assert_eq!(snapshot.events[1].name, "tests.cpu.inner");
    assert_eq!(snapshot.events[1].kind, CpuEventKind::SpanBegin);
    assert_eq!(snapshot.events[2].name, "tests.cpu.inner");
    assert_eq!(snapshot.events[2].kind, CpuEventKind::SpanEnd);
    assert_eq!(snapshot.events[3].name, "tests.cpu.outer");
    assert_eq!(snapshot.events[3].kind, CpuEventKind::SpanEnd);
  }

  #[test]
  fn cpu_counters_and_instants_enter_the_thread_local_buffer() {
    let collector = CpuCollector::with_thread_local_buffer_capacity(8);

    collector.record_counter("tests.cpu.counter", 42);
    collector.record_instant("tests.cpu.instant");

    let snapshot = collector.snapshot();

    assert_eq!(snapshot.events.len(), 2);
    assert_eq!(snapshot.events[0].kind, CpuEventKind::Counter { value: 42 });
    assert_eq!(snapshot.events[1].kind, CpuEventKind::Instant);
  }

  #[test]
  fn cpu_buffer_rollover_preserves_fixed_capacity_in_the_steady_path() {
    let collector = CpuCollector::with_thread_local_buffer_capacity(2);

    for _ in 0..5 {
      collector.record_instant("tests.cpu.rollover");
    }

    let snapshot = collector.snapshot();

    assert_eq!(snapshot.events.len(), 5);
    assert_eq!(snapshot.buffer_rollovers, 2);
    assert_eq!(snapshot.thread_local_buffer_capacity, 2);
    assert_eq!(snapshot.max_thread_local_buffer_capacity_observed, 2);
  }
}
