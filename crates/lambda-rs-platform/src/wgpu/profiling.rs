//! GPU timestamp-query helpers for Lambda's platform layer.
//!
//! This module keeps raw `wgpu` timestamp-query types confined to
//! `lambda-rs-platform` while exposing vendor-free helpers that higher layers
//! can use to detect support, allocate query/readback resources, encode query
//! resolution, and poll non-blocking readback completion.

use std::{
  ops::Range,
  sync::{
    Arc,
    Mutex,
  },
};

use super::{
  command::CommandEncoder,
  gpu::{
    Features,
    Gpu,
  },
};

/// The number of bytes occupied by one timestamp query result.
pub const TIMESTAMP_QUERY_BYTES_PER_RESULT: u64 = wgpu::QUERY_SIZE as u64;

/// The required alignment for query resolve destinations.
pub const TIMESTAMP_QUERY_RESOLVE_ALIGNMENT: u64 =
  wgpu::QUERY_RESOLVE_BUFFER_ALIGNMENT;

/// A snapshot of GPU timestamp-query support for a specific `Gpu`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GpuTimestampSupport {
  /// Whether the adapter supports timestamp queries at all.
  pub adapter_supports_timestamp_queries: bool,
  /// Whether the adapter supports pass-boundary timestamp writes.
  pub adapter_supports_pass_timestamp_writes: bool,
  /// Whether the current device was created with timestamp-query support.
  pub device_supports_timestamp_queries: bool,
  /// Whether the current device can write timestamps at pass boundaries.
  pub device_supports_pass_timestamp_writes: bool,
  /// The number of nanoseconds represented by one GPU timestamp tick.
  ///
  /// This value is zero when timestamp queries are unavailable on the current
  /// device.
  pub timestamp_period_ns: f32,
}

impl GpuTimestampSupport {
  /// Inspect timestamp-query support for an existing GPU device.
  ///
  /// # Arguments
  /// * `gpu` - The GPU device to inspect.
  ///
  /// # Returns
  /// A support snapshot describing adapter and device timestamp capability.
  pub fn for_gpu(gpu: &Gpu) -> Self {
    let adapter_features = gpu.adapter().features();
    let device_features = gpu.device().features();

    return Self {
      adapter_supports_timestamp_queries: adapter_features
        .contains(Features::TIMESTAMP_QUERY.to_wgpu()),
      adapter_supports_pass_timestamp_writes: adapter_features
        .contains(Features::TIMESTAMP_QUERY_INSIDE_PASSES.to_wgpu()),
      device_supports_timestamp_queries: device_features
        .contains(Features::TIMESTAMP_QUERY.to_wgpu()),
      device_supports_pass_timestamp_writes: device_features
        .contains(Features::TIMESTAMP_QUERY_INSIDE_PASSES.to_wgpu()),
      timestamp_period_ns: gpu.queue().get_timestamp_period(),
    };
  }

  /// Return whether pass-boundary GPU timestamps are fully available.
  ///
  /// # Returns
  /// `true` when the current device can record and resolve pass timestamps.
  pub fn is_available(self) -> bool {
    return self.device_supports_pass_timestamp_writes
      && self.timestamp_period_ns > 0.0;
  }
}

/// Errors returned by GPU timestamp-query helpers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GpuTimestampError {
  /// Timestamp queries are not available on the current GPU device.
  Unsupported {
    /// The reason timestamp queries could not be used.
    reason: GpuTimestampUnsupportedReason,
  },
  /// The requested query capacity was zero.
  InvalidQueryCapacity {
    /// The requested number of query slots.
    requested: u32,
  },
  /// The requested query range was invalid for the allocated batch.
  InvalidQueryRange {
    /// The first query index in the range.
    start: u32,
    /// The exclusive end query index in the range.
    end: u32,
    /// The total number of query slots in the batch.
    capacity: u32,
  },
  /// Readback was requested while a previous readback is still in flight.
  ReadbackPending,
  /// Readback was requested before query resolution was encoded.
  ReadbackNotEncoded,
  /// Polling the device for readback completion failed.
  PollFailed {
    /// The backend error as a human-readable string.
    message: String,
  },
  /// Mapping the readback buffer failed.
  MapFailed {
    /// The backend error as a human-readable string.
    message: String,
  },
}

impl GpuTimestampError {
  fn unsupported(reason: GpuTimestampUnsupportedReason) -> Self {
    return Self::Unsupported { reason };
  }
}

/// Reasons GPU timestamps may be unavailable on the current device.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GpuTimestampUnsupportedReason {
  /// The adapter does not support pass-boundary timestamp writes.
  AdapterFeatureUnavailable,
  /// The adapter supports timestamps, but the device was not created with them.
  DeviceFeatureDisabled,
}

#[derive(Debug)]
enum ReadbackState {
  Idle,
  Encoded {
    query_range: Range<u32>,
  },
  Pending {
    query_range: Range<u32>,
    callback_result: Arc<Mutex<Option<Result<(), String>>>>,
  },
}

/// Pass-boundary timestamp writes owned by a query batch.
#[derive(Clone, Copy, Debug)]
pub struct GpuRenderPassTimestampWrites<'a> {
  pub(crate) query_set: &'a wgpu::QuerySet,
  pub(crate) beginning_of_pass_write_index: Option<u32>,
  pub(crate) end_of_pass_write_index: Option<u32>,
}

/// A reusable query/readback batch for one GPU profiling frame.
#[derive(Debug)]
pub struct GpuTimestampQueryBatch {
  query_set: wgpu::QuerySet,
  resolve_buffer: wgpu::Buffer,
  readback_buffer: wgpu::Buffer,
  query_capacity: u32,
  timestamp_period_ns: f32,
  readback_state: Mutex<ReadbackState>,
}

impl GpuTimestampQueryBatch {
  /// Allocate a timestamp-query batch for one frame of GPU profiling.
  ///
  /// # Arguments
  /// * `gpu` - The GPU device that owns the query and readback resources.
  /// * `label` - A stable label used for the query and buffer resources.
  /// * `query_capacity` - The total number of timestamp slots to allocate.
  ///
  /// # Returns
  /// A reusable query batch when the device supports pass timestamps.
  ///
  /// # Errors
  /// Returns `GpuTimestampError` when timestamp support is unavailable or the
  /// requested capacity is invalid.
  pub fn new(
    gpu: &Gpu,
    label: &str,
    query_capacity: u32,
  ) -> Result<Self, GpuTimestampError> {
    if query_capacity == 0 {
      return Err(GpuTimestampError::InvalidQueryCapacity {
        requested: query_capacity,
      });
    }

    let support = GpuTimestampSupport::for_gpu(gpu);
    if !support.adapter_supports_pass_timestamp_writes {
      return Err(GpuTimestampError::unsupported(
        GpuTimestampUnsupportedReason::AdapterFeatureUnavailable,
      ));
    }

    if !support.is_available() {
      return Err(GpuTimestampError::unsupported(
        GpuTimestampUnsupportedReason::DeviceFeatureDisabled,
      ));
    }

    let buffer_size = query_range_byte_len(0..query_capacity)?;
    let query_set = gpu.device().create_query_set(&wgpu::QuerySetDescriptor {
      label: Some(label),
      ty: wgpu::QueryType::Timestamp,
      count: query_capacity,
    });
    let resolve_buffer = gpu.device().create_buffer(&wgpu::BufferDescriptor {
      label: Some(&format!("{}-resolve", label)),
      size: buffer_size,
      usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
      mapped_at_creation: false,
    });
    let readback_buffer = gpu.device().create_buffer(&wgpu::BufferDescriptor {
      label: Some(&format!("{}-readback", label)),
      size: buffer_size,
      usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
      mapped_at_creation: false,
    });

    return Ok(Self {
      query_set,
      resolve_buffer,
      readback_buffer,
      query_capacity,
      timestamp_period_ns: support.timestamp_period_ns,
      readback_state: Mutex::new(ReadbackState::Idle),
    });
  }

  /// Return the number of query slots in this batch.
  ///
  /// # Returns
  /// The total number of timestamp query indices available in the batch.
  pub fn query_capacity(&self) -> u32 {
    return self.query_capacity;
  }

  /// Return the timestamp period of the owning GPU queue in nanoseconds.
  ///
  /// # Returns
  /// The multiplier that converts raw timestamp ticks into nanoseconds.
  pub fn timestamp_period_ns(&self) -> f32 {
    return self.timestamp_period_ns;
  }

  /// Create pass-boundary timestamp writes for a render pass.
  ///
  /// # Arguments
  /// * `begin_query_index` - The query index that records pass start.
  /// * `end_query_index` - The query index that records pass end.
  ///
  /// # Returns
  /// A render-pass timestamp write configuration borrowing this batch.
  ///
  /// # Errors
  /// Returns `GpuTimestampError` when either index is outside the batch.
  pub fn render_pass_writes(
    &self,
    begin_query_index: u32,
    end_query_index: u32,
  ) -> Result<GpuRenderPassTimestampWrites<'_>, GpuTimestampError> {
    validate_query_index(begin_query_index, self.query_capacity)?;
    validate_query_index(end_query_index, self.query_capacity)?;

    return Ok(GpuRenderPassTimestampWrites {
      query_set: &self.query_set,
      beginning_of_pass_write_index: Some(begin_query_index),
      end_of_pass_write_index: Some(end_query_index),
    });
  }

  /// Encode query resolution and copy commands for a completed frame.
  ///
  /// # Arguments
  /// * `encoder` - The command encoder that will record the resolve/copy work.
  /// * `query_range` - The exclusive query range to resolve from this batch.
  ///
  /// # Returns
  /// `Ok(())` after the resolve/copy commands have been encoded.
  ///
  /// # Errors
  /// Returns `GpuTimestampError` when the range is invalid or when a previous
  /// readback has not completed yet.
  pub fn encode_resolve(
    &self,
    encoder: &mut CommandEncoder,
    query_range: Range<u32>,
  ) -> Result<(), GpuTimestampError> {
    validate_query_range(&query_range, self.query_capacity)?;

    let mut readback_state = self.readback_state.lock().expect("lock poisoned");
    if !matches!(*readback_state, ReadbackState::Idle) {
      return Err(GpuTimestampError::ReadbackPending);
    }

    let byte_len = query_range_byte_len(query_range.clone())?;
    encoder.resolve_query_set_raw(
      &self.query_set,
      query_range.clone(),
      &self.resolve_buffer,
      0,
    );
    encoder.copy_buffer_to_buffer_raw(
      &self.resolve_buffer,
      0,
      &self.readback_buffer,
      0,
      byte_len,
    );
    *readback_state = ReadbackState::Encoded { query_range };
    return Ok(());
  }

  /// Start a non-blocking readback for the most recently encoded query range.
  ///
  /// # Returns
  /// `Ok(())` after the readback request has been queued.
  ///
  /// # Errors
  /// Returns `GpuTimestampError` when no resolve has been encoded or a previous
  /// readback is still pending.
  pub fn begin_readback(&self) -> Result<(), GpuTimestampError> {
    let (query_range, callback_result) = {
      let mut readback_state =
        self.readback_state.lock().expect("lock poisoned");
      match &*readback_state {
        ReadbackState::Idle => {
          return Err(GpuTimestampError::ReadbackNotEncoded);
        }
        ReadbackState::Pending { .. } => {
          return Err(GpuTimestampError::ReadbackPending);
        }
        ReadbackState::Encoded { query_range } => {
          let callback_result = Arc::new(Mutex::new(None));
          let captured_callback_result = Arc::clone(&callback_result);
          let query_range = query_range.clone();
          let byte_len = query_range_byte_len(query_range.clone())?;
          self.readback_buffer.slice(0..byte_len).map_async(
            wgpu::MapMode::Read,
            move |result| {
              let mapped_result =
                result.map_err(|error| format!("{:?}", error));
              let mut slot =
                captured_callback_result.lock().expect("lock poisoned");
              *slot = Some(mapped_result);
            },
          );
          *readback_state = ReadbackState::Pending {
            query_range: query_range.clone(),
            callback_result: Arc::clone(&callback_result),
          };
          (query_range, callback_result)
        }
      }
    };

    let _ = query_range;
    let _ = callback_result;
    return Ok(());
  }

  /// Poll the device once and return completed timestamp values if ready.
  ///
  /// # Arguments
  /// * `gpu` - The GPU device that owns this batch.
  ///
  /// # Returns
  /// `Ok(Some(values))` when readback is complete, `Ok(None)` when it is still
  /// pending, or an error if polling or mapping failed.
  ///
  /// # Errors
  /// Returns `GpuTimestampError` when the device poll fails or buffer mapping
  /// completes with an error.
  pub fn try_readback(
    &self,
    gpu: &Gpu,
  ) -> Result<Option<Vec<u64>>, GpuTimestampError> {
    let poll_result =
      gpu.device().poll(wgpu::PollType::Poll).map_err(|error| {
        GpuTimestampError::PollFailed {
          message: format!("{:?}", error),
        }
      })?;
    let _ = poll_result;

    let (query_range, completed_result) = {
      let readback_state = self.readback_state.lock().expect("lock poisoned");
      match &*readback_state {
        ReadbackState::Idle | ReadbackState::Encoded { .. } => {
          return Ok(None);
        }
        ReadbackState::Pending {
          query_range,
          callback_result,
        } => (
          query_range.clone(),
          callback_result.lock().expect("lock poisoned").clone(),
        ),
      }
    };

    let Some(completed_result) = completed_result else {
      return Ok(None);
    };

    match completed_result {
      Ok(()) => {
        let byte_len = query_range_byte_len(query_range.clone())?;
        let mapped_view =
          self.readback_buffer.slice(0..byte_len).get_mapped_range();
        let values = mapped_view
          .chunks_exact(TIMESTAMP_QUERY_BYTES_PER_RESULT as usize)
          .map(|chunk| {
            let mut bytes = [0_u8; 8];
            bytes.copy_from_slice(chunk);
            u64::from_le_bytes(bytes)
          })
          .collect::<Vec<_>>();
        drop(mapped_view);
        self.readback_buffer.unmap();

        let mut readback_state =
          self.readback_state.lock().expect("lock poisoned");
        *readback_state = ReadbackState::Idle;
        return Ok(Some(values));
      }
      Err(message) => {
        let mut readback_state =
          self.readback_state.lock().expect("lock poisoned");
        *readback_state = ReadbackState::Idle;
        return Err(GpuTimestampError::MapFailed { message });
      }
    }
  }
}

fn validate_query_index(
  query_index: u32,
  query_capacity: u32,
) -> Result<(), GpuTimestampError> {
  if query_index >= query_capacity {
    return Err(GpuTimestampError::InvalidQueryRange {
      start: query_index,
      end: query_index.saturating_add(1),
      capacity: query_capacity,
    });
  }

  return Ok(());
}

fn validate_query_range(
  query_range: &Range<u32>,
  query_capacity: u32,
) -> Result<(), GpuTimestampError> {
  if query_range.is_empty() || query_range.end > query_capacity {
    return Err(GpuTimestampError::InvalidQueryRange {
      start: query_range.start,
      end: query_range.end,
      capacity: query_capacity,
    });
  }

  return Ok(());
}

fn query_range_byte_len(
  query_range: Range<u32>,
) -> Result<u64, GpuTimestampError> {
  let query_count = query_range.end.saturating_sub(query_range.start) as u64;
  let byte_len = query_count.saturating_mul(TIMESTAMP_QUERY_BYTES_PER_RESULT);
  return Ok(byte_len);
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::wgpu::{
    gpu::GpuBuilder,
    instance::InstanceBuilder,
  };

  fn create_test_gpu(required_features: Features) -> Option<Gpu> {
    let instance = InstanceBuilder::new()
      .with_label("profiling-gpu-test-instance")
      .build();
    return GpuBuilder::new()
      .with_label("profiling-gpu-test-device")
      .with_required_features(required_features)
      .build(&instance, None)
      .ok();
  }

  #[test]
  fn timestamp_support_reports_disabled_device_features_without_panic() {
    let Some(gpu) = create_test_gpu(Features::IMMEDIATES) else {
      eprintln!("Skipping timestamp support test: no compatible GPU adapter");
      return;
    };

    let support = GpuTimestampSupport::for_gpu(&gpu);
    assert!(!support.device_supports_timestamp_queries);
    assert!(!support.device_supports_pass_timestamp_writes);
    assert!(!support.is_available());
  }

  #[test]
  fn timestamp_query_batch_returns_actionable_error_when_disabled() {
    let Some(gpu) = create_test_gpu(Features::IMMEDIATES) else {
      eprintln!(
        "Skipping timestamp disabled allocation test: no compatible GPU adapter"
      );
      return;
    };

    let error = GpuTimestampQueryBatch::new(&gpu, "timestamp-disabled", 2)
      .expect_err("timestamp allocation should fail without enabled features");

    match error {
      GpuTimestampError::Unsupported { reason } => {
        assert!(
          reason == GpuTimestampUnsupportedReason::AdapterFeatureUnavailable
            || reason == GpuTimestampUnsupportedReason::DeviceFeatureDisabled
        );
      }
      other => panic!("unexpected timestamp error: {:?}", other),
    }
  }

  #[test]
  fn timestamp_query_batch_allocates_when_supported() {
    let required_features = Features::IMMEDIATES
      | Features::TIMESTAMP_QUERY
      | Features::TIMESTAMP_QUERY_INSIDE_PASSES;
    let Some(gpu) = create_test_gpu(required_features) else {
      eprintln!(
        "Skipping timestamp allocation success test: adapter or backend does \
not support pass timestamps"
      );
      return;
    };

    let support = GpuTimestampSupport::for_gpu(&gpu);
    assert!(support.device_supports_timestamp_queries);
    assert!(support.device_supports_pass_timestamp_writes);
    assert!(support.is_available());

    let batch = GpuTimestampQueryBatch::new(&gpu, "timestamp-enabled", 4)
      .expect("timestamp allocation should succeed");
    assert_eq!(batch.query_capacity(), 4);
    assert!(batch.timestamp_period_ns() > 0.0);

    let writes = batch
      .render_pass_writes(0, 1)
      .expect("render pass writes should fit in the batch");
    assert_eq!(writes.beginning_of_pass_write_index, Some(0));
    assert_eq!(writes.end_of_pass_write_index, Some(1));
    assert_eq!(batch.try_readback(&gpu).expect("poll should succeed"), None);
  }
}
