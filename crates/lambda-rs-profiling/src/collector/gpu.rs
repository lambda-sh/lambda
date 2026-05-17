#![allow(dead_code)]

use std::sync::{
  Arc,
  Mutex,
};

use lambda_platform::wgpu::{
  command::CommandEncoder as PlatformCommandEncoder,
  gpu::Gpu as PlatformGpu,
  profiling::{
    GpuRenderPassTimestampWrites,
    GpuTimestampQueryBatch,
    GpuTimestampSupport,
  },
};

/// Stable clock-domain label used for GPU summary scopes.
pub(crate) const GPU_CLOCK_DOMAIN: &str = "gpu";

/// Default number of frame slots retained in the GPU readback ring.
pub(crate) const DEFAULT_GPU_QUERY_RING_LEN: usize = 3;

/// Default maximum number of GPU spans reserved in one frame.
pub(crate) const DEFAULT_GPU_SCOPES_PER_FRAME: usize = 32;

/// Stores one resolved GPU span sample.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GpuSpan {
  pub(crate) name: &'static str,
  pub(crate) started_at_ns: u64,
  pub(crate) ended_at_ns: u64,
}

impl GpuSpan {
  /// Creates one resolved GPU span sample.
  ///
  /// # Arguments
  /// - `name`: Stable scope name for the resolved span.
  /// - `started_at_ns`: Submit-relative GPU timestamp in nanoseconds.
  /// - `ended_at_ns`: Submit-relative GPU timestamp in nanoseconds.
  ///
  /// # Returns
  /// Returns the resolved GPU span sample.
  pub(crate) fn new(
    name: &'static str,
    started_at_ns: u64,
    ended_at_ns: u64,
  ) -> Self {
    return Self {
      name,
      started_at_ns,
      ended_at_ns,
    };
  }

  /// Returns the resolved GPU span duration.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the span duration in nanoseconds.
  pub(crate) fn duration_ns(&self) -> u64 {
    return self.ended_at_ns.saturating_sub(self.started_at_ns);
  }
}

/// Describes why GPU collection is not active.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GpuCollectorDisabledReason {
  FeatureDisabled,
  UnsupportedAdapter,
  DeviceFeatureDisabled,
  QueryAllocationFailed,
}

/// Describes the current GPU collector mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GpuCollectorStatus {
  Disabled(GpuCollectorDisabledReason),
  PendingSupport,
  Ready,
}

/// One reserved GPU span within a frame slot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GpuScopeReservation {
  frame_index: usize,
  query_start_index: u32,
  query_end_index: u32,
  name: &'static str,
}

impl GpuScopeReservation {
  /// Returns the frame index associated with this reservation.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the zero-based frame index for the reservation.
  pub(crate) fn frame_index(&self) -> usize {
    return self.frame_index;
  }

  /// Returns the beginning query index used by the reservation.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the beginning timestamp query index.
  pub(crate) fn query_start_index(&self) -> u32 {
    return self.query_start_index;
  }

  /// Returns the ending query index used by the reservation.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the ending timestamp query index.
  pub(crate) fn query_end_index(&self) -> u32 {
    return self.query_end_index;
  }
}

/// One resolved GPU frame yielded after asynchronous readback completes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ResolvedGpuFrame {
  frame_index: usize,
  spans: Vec<GpuSpan>,
}

impl ResolvedGpuFrame {
  /// Creates one resolved GPU frame.
  ///
  /// # Arguments
  /// - `frame_index`: Zero-based frame index for the resolved readback.
  /// - `spans`: Resolved GPU spans for the frame.
  ///
  /// # Returns
  /// Returns the resolved GPU frame.
  pub(crate) fn new(frame_index: usize, spans: Vec<GpuSpan>) -> Self {
    return Self { frame_index, spans };
  }

  /// Returns the resolved frame index.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the zero-based frame index for the resolved spans.
  pub(crate) fn frame_index(&self) -> usize {
    return self.frame_index;
  }

  /// Returns the resolved GPU spans.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the resolved GPU span slice for the frame.
  pub(crate) fn spans(&self) -> &[GpuSpan] {
    return &self.spans;
  }
}

/// Owns the platform query batch needed by one reserved render-pass span.
#[derive(Clone, Debug)]
pub struct ReservedGpuRenderPassTimestamps {
  batch: Arc<GpuTimestampQueryBatch>,
  begin_query_index: u32,
  end_query_index: u32,
}

impl ReservedGpuRenderPassTimestamps {
  /// Creates one owned render-pass timestamp reservation.
  ///
  /// # Arguments
  /// - `batch`: Platform timestamp batch that owns the query set.
  /// - `begin_query_index`: Query index used for pass begin.
  /// - `end_query_index`: Query index used for pass end.
  ///
  /// # Returns
  /// Returns the owned timestamp reservation.
  pub(crate) fn new(
    batch: Arc<GpuTimestampQueryBatch>,
    begin_query_index: u32,
    end_query_index: u32,
  ) -> Self {
    return Self {
      batch,
      begin_query_index,
      end_query_index,
    };
  }

  /// Builds platform render-pass timestamp writes for a render pass lifetime.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns timestamp writes when the reserved indices fit the owning query
  /// batch, or `None` when the reservation is no longer valid.
  ///
  /// # Safety
  /// The caller MUST keep this reservation alive until the render pass that
  /// consumes the returned timestamp writes has ended. The returned descriptor
  /// borrows a query set owned by this reservation's query batch.
  pub unsafe fn timestamp_writes_for_pass_unchecked<'pass>(
    &self,
  ) -> Option<GpuRenderPassTimestampWrites<'pass>> {
    let timestamp_writes = self
      .batch
      .render_pass_writes(self.begin_query_index, self.end_query_index)
      .ok()?;

    return Some(std::mem::transmute::<
      GpuRenderPassTimestampWrites<'_>,
      GpuRenderPassTimestampWrites<'pass>,
    >(timestamp_writes));
  }
}

#[derive(Clone, Debug)]
struct RingSlot {
  assigned_frame_index: Option<usize>,
  reservations: Vec<GpuScopeReservation>,
  ready_spans: Option<Vec<GpuSpan>>,
  query_batch: Option<Arc<GpuTimestampQueryBatch>>,
}

impl RingSlot {
  fn new() -> Self {
    return Self {
      assigned_frame_index: None,
      reservations: Vec::new(),
      ready_spans: None,
      query_batch: None,
    };
  }

  fn clear_frame_data(&mut self) {
    self.assigned_frame_index = None;
    self.reservations.clear();
    self.ready_spans = None;
  }

  fn is_idle(&self) -> bool {
    return self.assigned_frame_index.is_none()
      && self.reservations.is_empty()
      && self.ready_spans.is_none();
  }
}

#[derive(Debug)]
struct GpuCollectorState {
  status: GpuCollectorStatus,
  timestamp_period_ns: f32,
  active_frame_index: usize,
  active_ring_index: usize,
  max_scopes_per_frame: usize,
  dropped_pending_frames: usize,
  ring_slots: Vec<RingSlot>,
}

impl GpuCollectorState {
  fn new(enabled: bool, ring_len: usize, max_scopes_per_frame: usize) -> Self {
    let ring_slot_count = ring_len.max(1);
    let status = if enabled {
      GpuCollectorStatus::PendingSupport
    } else {
      GpuCollectorStatus::Disabled(GpuCollectorDisabledReason::FeatureDisabled)
    };

    return Self {
      status,
      timestamp_period_ns: 0.0,
      active_frame_index: 0,
      active_ring_index: 0,
      max_scopes_per_frame: max_scopes_per_frame.max(1),
      dropped_pending_frames: 0,
      ring_slots: (0..ring_slot_count).map(|_| RingSlot::new()).collect(),
    };
  }
}

/// Collects asynchronous GPU profiling spans behind a frame ring.
#[derive(Debug)]
pub(crate) struct GpuCollector {
  state: Mutex<GpuCollectorState>,
}

impl GpuCollector {
  /// Creates a GPU collector with the default ring configuration.
  ///
  /// # Arguments
  /// - `enabled`: Whether GPU profiling was requested by the profiler builder.
  ///
  /// # Returns
  /// Returns a shared GPU collector.
  pub(crate) fn new(enabled: bool) -> Arc<Self> {
    return Self::with_ring_configuration(
      enabled,
      DEFAULT_GPU_QUERY_RING_LEN,
      DEFAULT_GPU_SCOPES_PER_FRAME,
    );
  }

  /// Creates a GPU collector with explicit ring sizing.
  ///
  /// # Arguments
  /// - `enabled`: Whether GPU profiling was requested by the profiler builder.
  /// - `ring_len`: Number of frame slots kept in the query/readback ring.
  /// - `max_scopes_per_frame`: Maximum GPU scopes reserved in one frame.
  ///
  /// # Returns
  /// Returns a shared GPU collector.
  pub(crate) fn with_ring_configuration(
    enabled: bool,
    ring_len: usize,
    max_scopes_per_frame: usize,
  ) -> Arc<Self> {
    return Arc::new(Self {
      state: Mutex::new(GpuCollectorState::new(
        enabled,
        ring_len,
        max_scopes_per_frame,
      )),
    });
  }

  /// Returns the current GPU collector status.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the current GPU collector status.
  pub(crate) fn status(&self) -> GpuCollectorStatus {
    let state = self
      .state
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());

    return state.status;
  }

  /// Applies a support snapshot from the platform layer.
  ///
  /// # Arguments
  /// - `support`: Support snapshot for the active GPU device.
  ///
  /// # Returns
  /// This method does not return a value.
  pub(crate) fn configure_support(&self, support: GpuTimestampSupport) {
    let mut state = self
      .state
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());

    if !Self::apply_support_snapshot(&mut state, support) {
      return;
    }

    state.status = GpuCollectorStatus::Ready;
    state.timestamp_period_ns = support.timestamp_period_ns;
  }

  /// Configures the collector from a real platform GPU and allocates query
  /// batches when timestamps are available.
  ///
  /// # Arguments
  /// - `gpu`: Platform GPU whose timestamp support and query resources should
  ///   be used for render-pass instrumentation.
  ///
  /// # Returns
  /// This method does not return a value. Unsupported devices leave the
  /// collector in CPU-only fallback mode.
  pub(crate) fn configure_for_gpu(&self, gpu: &PlatformGpu) {
    let support = GpuTimestampSupport::for_gpu(gpu);
    let mut state = self
      .state
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());

    if state.status == GpuCollectorStatus::Ready
      && state.ring_slots.iter().all(|slot| {
        return slot.query_batch.is_some();
      })
    {
      return;
    }

    if !Self::apply_support_snapshot(&mut state, support) {
      return;
    }

    let query_capacity = (state.max_scopes_per_frame.saturating_mul(2)) as u32;
    for slot_index in 0..state.ring_slots.len() {
      if state.ring_slots[slot_index].query_batch.is_some() {
        continue;
      }

      let label = format!("lambda-profiler-gpu-timestamps-{slot_index}");
      let Ok(batch) = GpuTimestampQueryBatch::new(gpu, &label, query_capacity)
      else {
        state.status = GpuCollectorStatus::Disabled(
          GpuCollectorDisabledReason::QueryAllocationFailed,
        );
        state.timestamp_period_ns = 0.0;
        return;
      };
      state.ring_slots[slot_index].query_batch = Some(Arc::new(batch));
    }

    state.status = GpuCollectorStatus::Ready;
    state.timestamp_period_ns = support.timestamp_period_ns;
  }

  /// Applies a timestamp support snapshot to collector state.
  ///
  /// # Arguments
  /// - `state`: Collector state to update.
  /// - `support`: Timestamp support snapshot from the platform layer.
  ///
  /// # Returns
  /// Returns `true` when GPU collection may continue and `false` when the
  /// collector should remain disabled or has fallen back to CPU-only mode.
  fn apply_support_snapshot(
    state: &mut GpuCollectorState,
    support: GpuTimestampSupport,
  ) -> bool {
    if matches!(
      state.status,
      GpuCollectorStatus::Disabled(GpuCollectorDisabledReason::FeatureDisabled)
    ) {
      return false;
    }

    if !support.adapter_supports_pass_timestamp_writes {
      state.status = GpuCollectorStatus::Disabled(
        GpuCollectorDisabledReason::UnsupportedAdapter,
      );
      state.timestamp_period_ns = 0.0;
      return false;
    }

    if !support.is_available() {
      state.status = GpuCollectorStatus::Disabled(
        GpuCollectorDisabledReason::DeviceFeatureDisabled,
      );
      state.timestamp_period_ns = 0.0;
      return false;
    }

    return true;
  }

  /// Reserves one GPU scope in the active frame.
  ///
  /// # Arguments
  /// - `name`: Stable scope name for the GPU span.
  ///
  /// # Returns
  /// Returns the reservation when GPU collection is active and capacity is
  /// available. Returns `None` when GPU profiling is disabled or the frame is
  /// already full.
  pub(crate) fn reserve_scope(
    &self,
    name: &'static str,
  ) -> Option<GpuScopeReservation> {
    let mut state = self
      .state
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());

    if state.status != GpuCollectorStatus::Ready {
      return None;
    }

    let active_frame_index = state.active_frame_index;
    let active_ring_index = state.active_ring_index;
    let max_scopes_per_frame = state.max_scopes_per_frame;
    let slot = &mut state.ring_slots[active_ring_index];

    if slot.reservations.len() >= max_scopes_per_frame {
      return None;
    }

    slot.assigned_frame_index = Some(active_frame_index);
    let query_start_index = (slot.reservations.len() * 2) as u32;
    let reservation = GpuScopeReservation {
      frame_index: active_frame_index,
      query_start_index,
      query_end_index: query_start_index + 1,
      name,
    };
    slot.reservations.push(reservation.clone());

    return Some(reservation);
  }

  /// Reserves timestamp writes for one render pass in the active frame.
  ///
  /// # Arguments
  /// - `name`: Stable GPU scope name for the render pass.
  ///
  /// # Returns
  /// Returns an owned timestamp reservation when the collector is ready and
  /// the active frame has a platform query batch.
  pub(crate) fn reserve_render_pass_timestamps(
    &self,
    name: &'static str,
  ) -> Option<ReservedGpuRenderPassTimestamps> {
    let reservation = self.reserve_scope(name)?;
    let state = self
      .state
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());

    let slot = state.ring_slots.iter().find(|slot| {
      return slot.assigned_frame_index == Some(reservation.frame_index);
    })?;
    let batch = slot.query_batch.as_ref()?.clone();

    return Some(ReservedGpuRenderPassTimestamps::new(
      batch,
      reservation.query_start_index,
      reservation.query_end_index,
    ));
  }

  /// Encodes query resolution for the active GPU profiling frame.
  ///
  /// # Arguments
  /// - `encoder`: Platform command encoder receiving query resolve commands.
  ///
  /// # Returns
  /// This method does not return a value. Resolve failures disable the active
  /// frame's GPU data without affecting command submission.
  pub(crate) fn prepare_frame_submission(
    &self,
    encoder: &mut PlatformCommandEncoder,
  ) {
    let (batch, query_count) = {
      let mut state = self
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

      if state.status != GpuCollectorStatus::Ready {
        return;
      }

      let active_ring_index = state.active_ring_index;
      let slot = &mut state.ring_slots[active_ring_index];
      if slot.reservations.is_empty() {
        return;
      }

      let Some(batch) = slot.query_batch.as_ref().cloned() else {
        slot.clear_frame_data();
        return;
      };

      (batch, (slot.reservations.len() * 2) as u32)
    };

    if batch.encode_resolve(encoder, 0..query_count).is_err() {
      let mut state = self
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
      let active_ring_index = state.active_ring_index;
      state.ring_slots[active_ring_index].clear_frame_data();
    }
  }

  /// Marks the active GPU profiling frame submitted and polls readbacks.
  ///
  /// # Arguments
  /// - `gpu`: Platform GPU used to poll asynchronous readback completion.
  ///
  /// # Returns
  /// Returns any frames whose GPU timestamp readback completed without
  /// blocking the active render thread. The active ring slot advances on the
  /// next profiler frame flush so CPU and GPU frame boundaries stay aligned.
  pub(crate) fn complete_frame_submission(
    &self,
    gpu: &PlatformGpu,
  ) -> Vec<ResolvedGpuFrame> {
    self.begin_active_readback();
    return self.poll_readbacks(gpu);
  }

  /// Advances the active frame without blocking for pending readback.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns any resolved GPU frames that became available before the ring
  /// advanced.
  pub(crate) fn advance_frame(&self) -> Vec<ResolvedGpuFrame> {
    let mut state = self
      .state
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());

    let mut resolved_frames = Vec::new();
    for slot in &mut state.ring_slots {
      if let (Some(frame_index), Some(spans)) =
        (slot.assigned_frame_index, slot.ready_spans.take())
      {
        resolved_frames.push(ResolvedGpuFrame::new(frame_index, spans));
        slot.clear_frame_data();
      }
    }

    if state.status != GpuCollectorStatus::Ready {
      return resolved_frames;
    }

    state.active_frame_index += 1;
    state.active_ring_index =
      (state.active_ring_index + 1) % state.ring_slots.len();
    let active_ring_index = state.active_ring_index;

    let next_slot_is_idle = {
      let next_slot = &state.ring_slots[active_ring_index];
      next_slot.is_idle()
    };
    if !next_slot_is_idle {
      state.dropped_pending_frames += 1;
      state.ring_slots[active_ring_index].clear_frame_data();
    }

    return resolved_frames;
  }

  /// Starts readback for the active frame after its command buffer submits.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// This method does not return a value. If no resolve was encoded, the
  /// active frame is left in place for normal ring advancement.
  fn begin_active_readback(&self) {
    let batch = {
      let state = self
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

      if state.status != GpuCollectorStatus::Ready {
        return;
      }

      state.ring_slots[state.active_ring_index]
        .query_batch
        .as_ref()
        .cloned()
    };

    let Some(batch) = batch else {
      return;
    };
    let _ = batch.begin_readback();
  }

  /// Polls all pending readbacks once without blocking.
  ///
  /// # Arguments
  /// - `gpu`: Platform GPU used for device polling.
  ///
  /// # Returns
  /// Returns frames whose timestamp readback completed during this poll.
  fn poll_readbacks(&self, gpu: &PlatformGpu) -> Vec<ResolvedGpuFrame> {
    let pending_slots = {
      let state = self
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

      if state.status != GpuCollectorStatus::Ready {
        return Vec::new();
      }

      state
        .ring_slots
        .iter()
        .enumerate()
        .filter_map(|(slot_index, slot)| {
          let frame_index = slot.assigned_frame_index?;
          let batch = slot.query_batch.as_ref()?.clone();

          return Some((
            slot_index,
            frame_index,
            slot.reservations.clone(),
            batch,
          ));
        })
        .collect::<Vec<_>>()
    };

    let mut resolved_frames = Vec::new();
    let mut slots_to_clear = Vec::new();

    for (slot_index, frame_index, reservations, batch) in pending_slots {
      let readback = batch.try_readback(gpu);
      let Ok(Some(timestamp_values)) = readback else {
        if readback.is_err() {
          slots_to_clear.push(slot_index);
        }
        continue;
      };

      let Some(spans) = build_spans_from_timestamp_values(
        &reservations,
        &timestamp_values,
        batch.timestamp_period_ns(),
      ) else {
        slots_to_clear.push(slot_index);
        continue;
      };

      slots_to_clear.push(slot_index);
      resolved_frames.push(ResolvedGpuFrame::new(frame_index, spans));
    }

    if !slots_to_clear.is_empty() {
      let mut state = self
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

      for slot_index in slots_to_clear {
        if let Some(slot) = state.ring_slots.get_mut(slot_index) {
          slot.clear_frame_data();
        }
      }
    }

    return resolved_frames;
  }

  /// Injects resolved raw timestamp ticks for one completed frame.
  ///
  /// This method models the non-blocking readback step used by later render
  /// instrumentation without requiring an actual GPU in unit tests.
  ///
  /// # Arguments
  /// - `frame_index`: Zero-based frame index associated with the readback.
  /// - `timestamp_ticks`: Raw begin/end timestamp pairs in GPU ticks.
  ///
  /// # Returns
  /// Returns `true` when the frame was matched and stored for later export.
  pub(crate) fn store_resolved_ticks(
    &self,
    frame_index: usize,
    timestamp_ticks: &[(u64, u64)],
  ) -> bool {
    let mut state = self
      .state
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());

    if state.status != GpuCollectorStatus::Ready {
      return false;
    }

    let timestamp_period_ns = state.timestamp_period_ns;

    for slot in &mut state.ring_slots {
      if slot.assigned_frame_index != Some(frame_index) {
        continue;
      }

      let Some(spans) = build_spans_from_timestamp_tick_pairs(
        &slot.reservations,
        timestamp_ticks,
        timestamp_period_ns,
      ) else {
        return false;
      };

      slot.ready_spans = Some(spans);

      return true;
    }

    return false;
  }

  #[cfg(test)]
  pub(crate) fn active_frame_index(&self) -> usize {
    let state = self
      .state
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());

    return state.active_frame_index;
  }

  #[cfg(test)]
  pub(crate) fn dropped_pending_frames(&self) -> usize {
    let state = self
      .state
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());

    return state.dropped_pending_frames;
  }
}

/// Converts raw timestamp values into resolved GPU spans.
///
/// # Arguments
/// - `reservations`: Scope reservations associated with the timestamp values.
/// - `timestamp_values`: Flat begin/end timestamp values read from a query
///   batch.
/// - `timestamp_period_ns`: Multiplier that converts ticks into nanoseconds.
///
/// # Returns
/// Returns resolved spans when the value count matches the reservations.
fn build_spans_from_timestamp_values(
  reservations: &[GpuScopeReservation],
  timestamp_values: &[u64],
  timestamp_period_ns: f32,
) -> Option<Vec<GpuSpan>> {
  if timestamp_values.len() != reservations.len().saturating_mul(2) {
    return None;
  }

  let timestamp_pairs = timestamp_values
    .chunks_exact(2)
    .map(|chunk| {
      return (chunk[0], chunk[1]);
    })
    .collect::<Vec<_>>();

  return build_spans_from_timestamp_tick_pairs(
    reservations,
    &timestamp_pairs,
    timestamp_period_ns,
  );
}

/// Converts raw begin/end timestamp pairs into resolved GPU spans.
///
/// # Arguments
/// - `reservations`: Scope reservations associated with the timestamp pairs.
/// - `timestamp_ticks`: Raw begin/end timestamp pairs in GPU ticks.
/// - `timestamp_period_ns`: Multiplier that converts ticks into nanoseconds.
///
/// # Returns
/// Returns resolved spans when the pair count matches the reservations.
fn build_spans_from_timestamp_tick_pairs(
  reservations: &[GpuScopeReservation],
  timestamp_ticks: &[(u64, u64)],
  timestamp_period_ns: f32,
) -> Option<Vec<GpuSpan>> {
  if reservations.len() != timestamp_ticks.len() {
    return None;
  }

  return Some(
    reservations
      .iter()
      .zip(timestamp_ticks.iter())
      .map(|(reservation, (begin_ticks, end_ticks))| {
        let started_at_ns =
          ((*begin_ticks as f64) * timestamp_period_ns as f64) as u64;
        let ended_at_ns =
          ((*end_ticks as f64) * timestamp_period_ns as f64) as u64;

        return GpuSpan::new(reservation.name, started_at_ns, ended_at_ns);
      })
      .collect(),
  );
}

#[cfg(test)]
mod tests {
  use lambda_platform::wgpu::profiling::GpuTimestampSupport;

  use super::{
    GpuCollector,
    GpuCollectorDisabledReason,
    GpuCollectorStatus,
  };

  fn unsupported_support() -> GpuTimestampSupport {
    return GpuTimestampSupport {
      adapter_supports_timestamp_queries: false,
      adapter_supports_pass_timestamp_writes: false,
      device_supports_timestamp_queries: false,
      device_supports_pass_timestamp_writes: false,
      timestamp_period_ns: 0.0,
    };
  }

  fn supported_snapshot() -> GpuTimestampSupport {
    return GpuTimestampSupport {
      adapter_supports_timestamp_queries: true,
      adapter_supports_pass_timestamp_writes: true,
      device_supports_timestamp_queries: true,
      device_supports_pass_timestamp_writes: true,
      timestamp_period_ns: 1.0,
    };
  }

  #[test]
  fn falls_back_to_cpu_only_mode_when_gpu_timestamps_are_unavailable() {
    let collector = GpuCollector::new(true);

    collector.configure_support(unsupported_support());

    assert_eq!(
      collector.status(),
      GpuCollectorStatus::Disabled(
        GpuCollectorDisabledReason::UnsupportedAdapter
      )
    );
    assert!(collector.reserve_scope("tests.gpu.scope").is_none());
  }

  #[test]
  fn query_ring_advances_without_blocking_the_active_frame() {
    let collector = GpuCollector::with_ring_configuration(true, 2, 2);

    collector.configure_support(supported_snapshot());

    let reservation = collector.reserve_scope("tests.gpu.frame0").unwrap();
    assert_eq!(reservation.frame_index(), 0);
    collector.advance_frame();
    assert_eq!(collector.active_frame_index(), 1);

    let reservation = collector.reserve_scope("tests.gpu.frame1").unwrap();
    assert_eq!(reservation.frame_index(), 1);
    collector.advance_frame();

    assert_eq!(collector.active_frame_index(), 2);
    assert_eq!(collector.dropped_pending_frames(), 1);
  }

  #[test]
  fn resolved_gpu_frames_are_returned_after_the_next_frame_advance() {
    let collector = GpuCollector::with_ring_configuration(true, 3, 2);

    collector.configure_support(supported_snapshot());

    let reservation = collector.reserve_scope("tests.gpu.ready").unwrap();
    assert_eq!(reservation.query_start_index(), 0);
    assert_eq!(reservation.query_end_index(), 1);
    collector.advance_frame();

    assert!(collector.store_resolved_ticks(0, &[(10, 25)]));

    let resolved_frames = collector.advance_frame();

    assert_eq!(resolved_frames.len(), 1);
    assert_eq!(resolved_frames[0].frame_index(), 0);
    assert_eq!(resolved_frames[0].spans()[0].name, "tests.gpu.ready");
    assert_eq!(resolved_frames[0].spans()[0].started_at_ns, 10);
    assert_eq!(resolved_frames[0].spans()[0].ended_at_ns, 25);
  }
}
