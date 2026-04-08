use std::collections::{
  BTreeMap,
  HashMap,
};

use crate::collector::cpu::{
  CpuEvent,
  CpuEventKind,
};

/// Stable clock-domain label used for CPU summary scopes.
pub(crate) const CPU_CLOCK_DOMAIN: &str = "cpu";

const SUMMARY_SCHEMA_VERSION: &str = "1";
const TRACE_SCHEMA_VERSION: &str = "1";

/// Stores one completed frame retained by the profiler.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CompletedFrame {
  frame_index: usize,
  frame_duration_ns: u64,
  cpu_events: Vec<CpuEvent>,
}

impl CompletedFrame {
  /// Creates a completed CPU frame.
  ///
  /// # Arguments
  /// - `frame_index`: Zero-based index assigned during frame flushing.
  /// - `frame_duration_ns`: Total completed frame duration in nanoseconds.
  /// - `cpu_events`: Ordered CPU events drained for the completed frame.
  ///
  /// # Returns
  /// Returns the retained completed frame record.
  pub(crate) fn cpu(
    frame_index: usize,
    frame_duration_ns: u64,
    cpu_events: Vec<CpuEvent>,
  ) -> Self {
    return Self {
      frame_index,
      frame_duration_ns,
      cpu_events,
    };
  }

  /// Returns the retained frame index.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the zero-based frame index for this completed frame.
  pub(crate) fn frame_index(&self) -> usize {
    return self.frame_index;
  }

  /// Returns the retained frame duration.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the completed frame duration in nanoseconds.
  pub(crate) fn frame_duration_ns(&self) -> u64 {
    return self.frame_duration_ns;
  }

  /// Returns the retained CPU events for this completed frame.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the ordered CPU event slice for this frame.
  pub(crate) fn cpu_events(&self) -> &[CpuEvent] {
    return &self.cpu_events;
  }
}

/// Aggregated metrics for one named scope in a summary report.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SummaryScope {
  name: String,
  clock_domain: String,
  count: u64,
  total_ns: u64,
  mean_ns: u64,
  p50_ns: u64,
  p95_ns: u64,
  max_ns: u64,
}

impl SummaryScope {
  /// Returns the stable scope name.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the scope name recorded for this summary entry.
  pub fn name(&self) -> &str {
    return &self.name;
  }

  /// Returns the scope clock domain.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the stable clock-domain label for this scope summary.
  pub fn clock_domain(&self) -> &str {
    return &self.clock_domain;
  }

  /// Returns the retained sample count.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the number of retained samples aggregated into this scope.
  pub fn count(&self) -> u64 {
    return self.count;
  }

  /// Returns the total retained duration.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the total retained duration in nanoseconds.
  pub fn total_ns(&self) -> u64 {
    return self.total_ns;
  }

  /// Returns the arithmetic mean duration.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the mean retained duration in nanoseconds.
  pub fn mean_ns(&self) -> u64 {
    return self.mean_ns;
  }

  /// Returns the retained p50 duration.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the p50 retained duration in nanoseconds.
  pub fn p50_ns(&self) -> u64 {
    return self.p50_ns;
  }

  /// Returns the retained p95 duration.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the p95 retained duration in nanoseconds.
  pub fn p95_ns(&self) -> u64 {
    return self.p95_ns;
  }

  /// Returns the maximum retained duration.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the maximum retained duration in nanoseconds.
  pub fn max_ns(&self) -> u64 {
    return self.max_ns;
  }
}

/// Aggregated profiling data for retained completed frames.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SummaryReport {
  run_name: String,
  crate_version: String,
  generated_at_utc: String,
  scopes: Vec<SummaryScope>,
  frame_count: u64,
  frame_p50_ns: u64,
  frame_p95_ns: u64,
  frame_max_ns: u64,
}

impl SummaryReport {
  /// Builds a summary report from retained completed frames.
  ///
  /// # Arguments
  /// - `run_name`: Stable run name configured for the profiler.
  /// - `generated_at_utc`: UTC timestamp embedded in the report.
  /// - `frames`: Retained completed frames used for aggregation.
  ///
  /// # Returns
  /// Returns a stable summary report computed from the retained frames.
  pub(crate) fn from_completed_frames(
    run_name: &str,
    generated_at_utc: String,
    frames: &[CompletedFrame],
  ) -> Self {
    let frame_durations: Vec<u64> = frames
      .iter()
      .map(CompletedFrame::frame_duration_ns)
      .collect();
    let scopes = build_cpu_scope_summaries(frames);

    return Self {
      run_name: run_name.to_owned(),
      crate_version: env!("CARGO_PKG_VERSION").to_owned(),
      generated_at_utc,
      scopes,
      frame_count: frame_durations.len() as u64,
      frame_p50_ns: percentile_ns(&frame_durations, 50),
      frame_p95_ns: percentile_ns(&frame_durations, 95),
      frame_max_ns: frame_durations.iter().copied().max().unwrap_or_default(),
    };
  }

  /// Returns the stable schema version for summary export.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the current summary schema version.
  pub fn schema_version(&self) -> &'static str {
    return SUMMARY_SCHEMA_VERSION;
  }

  /// Returns the profiler run name used for this report.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the stable run name stored in the report.
  pub fn run_name(&self) -> &str {
    return &self.run_name;
  }

  /// Returns the profiling crate version stored in this report.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the profiling crate version string.
  pub fn crate_version(&self) -> &str {
    return &self.crate_version;
  }

  /// Returns the UTC timestamp stored in this report.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the generated-at timestamp in UTC.
  pub fn generated_at_utc(&self) -> &str {
    return &self.generated_at_utc;
  }

  /// Returns the stable clock domains present in this report.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the ordered clock-domain list for this report.
  pub fn clock_domains(&self) -> [&str; 1] {
    return [CPU_CLOCK_DOMAIN];
  }

  /// Returns the retained scope summaries.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the stable ordered scope summaries stored in the report.
  pub fn scopes(&self) -> &[SummaryScope] {
    return &self.scopes;
  }

  /// Returns the number of retained completed frames.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the frame count used to compute frame metrics.
  pub fn frame_count(&self) -> u64 {
    return self.frame_count;
  }

  /// Returns the retained frame p50 duration.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the p50 retained frame duration in nanoseconds.
  pub fn frame_p50_ns(&self) -> u64 {
    return self.frame_p50_ns;
  }

  /// Returns the retained frame p95 duration.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the p95 retained frame duration in nanoseconds.
  pub fn frame_p95_ns(&self) -> u64 {
    return self.frame_p95_ns;
  }

  /// Returns the retained maximum frame duration.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the maximum retained frame duration in nanoseconds.
  pub fn frame_max_ns(&self) -> u64 {
    return self.frame_max_ns;
  }

  /// Serializes this summary report into stable JSON bytes.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the pretty-printed JSON bytes for the summary report.
  ///
  /// # Errors
  /// Returns [`crate::ExportError::Serialize`] when summary serialization is
  /// unavailable or fails.
  pub(crate) fn to_json_bytes(&self) -> Result<Vec<u8>, crate::ExportError> {
    #[cfg(feature = "export-summary")]
    {
      use lambda_platform::profiling_support::deps::serde_json::{
        self,
        Map,
        Number,
        Value,
      };

      let scope_values = self
        .scopes
        .iter()
        .map(|scope| {
          let mut scope_object = Map::new();

          scope_object
            .insert("name".to_owned(), Value::String(scope.name.clone()));
          scope_object.insert(
            "clock_domain".to_owned(),
            Value::String(scope.clock_domain.clone()),
          );
          scope_object.insert(
            "count".to_owned(),
            Value::Number(Number::from(scope.count)),
          );
          scope_object.insert(
            "total_ns".to_owned(),
            Value::Number(Number::from(scope.total_ns)),
          );
          scope_object.insert(
            "mean_ns".to_owned(),
            Value::Number(Number::from(scope.mean_ns)),
          );
          scope_object.insert(
            "p50_ns".to_owned(),
            Value::Number(Number::from(scope.p50_ns)),
          );
          scope_object.insert(
            "p95_ns".to_owned(),
            Value::Number(Number::from(scope.p95_ns)),
          );
          scope_object.insert(
            "max_ns".to_owned(),
            Value::Number(Number::from(scope.max_ns)),
          );

          return Value::Object(scope_object);
        })
        .collect();

      let mut root_object = Map::new();

      root_object.insert(
        "schema_version".to_owned(),
        Value::String(self.schema_version().to_owned()),
      );
      root_object
        .insert("run_name".to_owned(), Value::String(self.run_name.clone()));
      root_object.insert(
        "crate_version".to_owned(),
        Value::String(self.crate_version.clone()),
      );
      root_object.insert(
        "clock_domains".to_owned(),
        Value::Array(
          self
            .clock_domains()
            .iter()
            .map(|clock_domain| Value::String((*clock_domain).to_owned()))
            .collect(),
        ),
      );
      root_object.insert(
        "generated_at_utc".to_owned(),
        Value::String(self.generated_at_utc.clone()),
      );
      root_object.insert("scopes".to_owned(), Value::Array(scope_values));
      root_object.insert(
        "frame_count".to_owned(),
        Value::Number(Number::from(self.frame_count)),
      );
      root_object.insert(
        "frame_p50_ns".to_owned(),
        Value::Number(Number::from(self.frame_p50_ns)),
      );
      root_object.insert(
        "frame_p95_ns".to_owned(),
        Value::Number(Number::from(self.frame_p95_ns)),
      );
      root_object.insert(
        "frame_max_ns".to_owned(),
        Value::Number(Number::from(self.frame_max_ns)),
      );

      return serde_json::to_vec_pretty(&Value::Object(root_object)).map_err(
        |_| {
          return crate::ExportError::Serialize {
            export_name: "summary",
          };
        },
      );
    }

    #[cfg(not(feature = "export-summary"))]
    {
      return Err(crate::ExportError::Unavailable {
        export_name: "summary",
      });
    }
  }
}

/// Stores one ordered profiling event exported by a trace report.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TraceEvent {
  name: String,
  kind: String,
  clock_domain: String,
  timestamp_ns: u64,
  thread_id: Option<String>,
  frame_index: u64,
  value: Option<u64>,
}

impl TraceEvent {
  /// Returns the stable event name.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the recorded event name.
  pub fn name(&self) -> &str {
    return &self.name;
  }

  /// Returns the stable event kind label.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns one of `span_begin`, `span_end`, `counter`, or `instant`.
  pub fn kind(&self) -> &str {
    return &self.kind;
  }

  /// Returns the stable clock-domain label for this event.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the event clock-domain label.
  pub fn clock_domain(&self) -> &str {
    return &self.clock_domain;
  }

  /// Returns the collector-relative event timestamp.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the event timestamp in nanoseconds.
  pub fn timestamp_ns(&self) -> u64 {
    return self.timestamp_ns;
  }

  /// Returns the CPU thread identifier string when one exists.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the thread identifier string for CPU events, or `None` for
  /// clock domains that do not carry thread affinity.
  pub fn thread_id(&self) -> Option<&str> {
    return self.thread_id.as_deref();
  }

  /// Returns the zero-based frame index containing this event.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the zero-based frame index for the event.
  pub fn frame_index(&self) -> u64 {
    return self.frame_index;
  }

  /// Returns the counter value when this event stores one.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the retained counter value for `counter` events, or `None`
  /// otherwise.
  pub fn value(&self) -> Option<u64> {
    return self.value;
  }
}

/// Ordered profiling events for a completed run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TraceReport {
  run_name: String,
  generated_at_utc: String,
  events: Vec<TraceEvent>,
}

impl TraceReport {
  /// Builds a trace report from retained completed frames.
  ///
  /// # Arguments
  /// - `run_name`: Stable run name configured for the profiler.
  /// - `generated_at_utc`: UTC timestamp embedded in the report.
  /// - `frames`: Retained completed frames used for export.
  ///
  /// # Returns
  /// Returns an ordered trace report over the retained frames.
  pub(crate) fn from_completed_frames(
    run_name: &str,
    generated_at_utc: String,
    frames: &[CompletedFrame],
  ) -> Self {
    return Self {
      run_name: run_name.to_owned(),
      generated_at_utc,
      events: build_trace_events(frames),
    };
  }

  /// Returns the stable schema version for trace export.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the current trace schema version.
  pub fn schema_version(&self) -> &'static str {
    return TRACE_SCHEMA_VERSION;
  }

  /// Returns the profiler run name used for this report.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the stable run name stored in the report.
  pub fn run_name(&self) -> &str {
    return &self.run_name;
  }

  /// Returns the UTC timestamp stored in this report.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the generated-at timestamp in UTC.
  pub fn generated_at_utc(&self) -> &str {
    return &self.generated_at_utc;
  }

  /// Returns the ordered trace events stored in this report.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the ordered trace-event slice for this report.
  pub fn events(&self) -> &[TraceEvent] {
    return &self.events;
  }

  /// Serializes this trace report into stable JSON bytes.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the pretty-printed JSON bytes for the trace report.
  ///
  /// # Errors
  /// Returns [`crate::ExportError::Serialize`] when trace serialization is
  /// unavailable or fails.
  pub(crate) fn to_json_bytes(&self) -> Result<Vec<u8>, crate::ExportError> {
    #[cfg(feature = "export-trace")]
    {
      use lambda_platform::profiling_support::deps::serde_json::{
        self,
        Map,
        Number,
        Value,
      };

      let event_values = self
        .events
        .iter()
        .map(|event| {
          let mut event_object = Map::new();

          event_object
            .insert("name".to_owned(), Value::String(event.name.clone()));
          event_object
            .insert("kind".to_owned(), Value::String(event.kind.clone()));
          event_object.insert(
            "clock_domain".to_owned(),
            Value::String(event.clock_domain.clone()),
          );
          event_object.insert(
            "timestamp_ns".to_owned(),
            Value::Number(Number::from(event.timestamp_ns)),
          );
          event_object.insert(
            "frame_index".to_owned(),
            Value::Number(Number::from(event.frame_index)),
          );

          if let Some(thread_id) = &event.thread_id {
            event_object
              .insert("thread_id".to_owned(), Value::String(thread_id.clone()));
          }

          if let Some(value) = event.value {
            event_object
              .insert("value".to_owned(), Value::Number(Number::from(value)));
          }

          return Value::Object(event_object);
        })
        .collect();

      let mut root_object = Map::new();

      root_object.insert(
        "schema_version".to_owned(),
        Value::String(self.schema_version().to_owned()),
      );
      root_object
        .insert("run_name".to_owned(), Value::String(self.run_name.clone()));
      root_object.insert(
        "generated_at_utc".to_owned(),
        Value::String(self.generated_at_utc.clone()),
      );
      root_object.insert("events".to_owned(), Value::Array(event_values));

      return serde_json::to_vec_pretty(&Value::Object(root_object)).map_err(
        |_| {
          return crate::ExportError::Serialize {
            export_name: "trace",
          };
        },
      );
    }

    #[cfg(not(feature = "export-trace"))]
    {
      return Err(crate::ExportError::Unavailable {
        export_name: "trace",
      });
    }
  }
}

/// Outcome of comparing a current profiling run against a baseline.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ComparisonReport {
  matched_scope_count: usize,
  violation_count: usize,
}

/// Builds stable CPU scope summaries from retained completed frames.
///
/// # Arguments
/// - `frames`: Retained completed frames used for aggregation.
///
/// # Returns
/// Returns the ordered CPU scope summaries computed from the retained frames.
fn build_cpu_scope_summaries(frames: &[CompletedFrame]) -> Vec<SummaryScope> {
  let mut samples_by_scope: BTreeMap<(&'static str, &'static str), Vec<u64>> =
    BTreeMap::new();

  for frame in frames {
    let mut active_spans_by_thread: HashMap<
      std::thread::ThreadId,
      Vec<(&'static str, u64)>,
    > = HashMap::new();

    for event in frame.cpu_events() {
      match event.kind {
        CpuEventKind::SpanBegin => {
          active_spans_by_thread
            .entry(event.thread_id)
            .or_default()
            .push((event.name, event.timestamp_ns));
        }
        CpuEventKind::SpanEnd => {
          let Some(active_spans) =
            active_spans_by_thread.get_mut(&event.thread_id)
          else {
            continue;
          };
          let Some((started_name, started_at_ns)) = active_spans.pop() else {
            continue;
          };
          let duration_ns = event.timestamp_ns.saturating_sub(started_at_ns);

          samples_by_scope
            .entry((started_name, CPU_CLOCK_DOMAIN))
            .or_default()
            .push(duration_ns);
        }
        CpuEventKind::Counter { .. } | CpuEventKind::Instant => {}
      }
    }
  }

  return samples_by_scope
    .into_iter()
    .map(|((name, clock_domain), samples)| {
      return build_scope_summary(name, clock_domain, &samples);
    })
    .collect();
}

/// Builds ordered trace events from retained completed frames.
///
/// # Arguments
/// - `frames`: Retained completed frames used for export.
///
/// # Returns
/// Returns the ordered trace events derived from the retained frames.
fn build_trace_events(frames: &[CompletedFrame]) -> Vec<TraceEvent> {
  let mut trace_events = Vec::new();

  for frame in frames {
    for event in frame.cpu_events() {
      let (kind, value) = match event.kind {
        CpuEventKind::SpanBegin => ("span_begin", None),
        CpuEventKind::SpanEnd => ("span_end", None),
        CpuEventKind::Counter { value } => ("counter", Some(value)),
        CpuEventKind::Instant => ("instant", None),
      };

      trace_events.push(TraceEvent {
        name: event.name.to_owned(),
        kind: kind.to_owned(),
        clock_domain: CPU_CLOCK_DOMAIN.to_owned(),
        timestamp_ns: event.timestamp_ns,
        thread_id: Some(format!("{:?}", event.thread_id)),
        frame_index: frame.frame_index() as u64,
        value,
      });
    }
  }

  return trace_events;
}

/// Builds one stable scope summary from retained duration samples.
///
/// # Arguments
/// - `name`: Stable scope name.
/// - `clock_domain`: Stable clock-domain name.
/// - `samples`: Retained duration samples in nanoseconds.
///
/// # Returns
/// Returns the aggregated summary for the provided scope samples.
fn build_scope_summary(
  name: &'static str,
  clock_domain: &'static str,
  samples: &[u64],
) -> SummaryScope {
  let count = samples.len() as u64;
  let total_ns = samples.iter().copied().sum::<u64>();
  let mean_ns = if count == 0 { 0 } else { total_ns / count };

  return SummaryScope {
    name: name.to_owned(),
    clock_domain: clock_domain.to_owned(),
    count,
    total_ns,
    mean_ns,
    p50_ns: percentile_ns(samples, 50),
    p95_ns: percentile_ns(samples, 95),
    max_ns: samples.iter().copied().max().unwrap_or_default(),
  };
}

/// Calculates a percentile using nearest-rank semantics.
///
/// # Arguments
/// - `samples`: Retained input samples in nanoseconds.
/// - `percentile`: Requested percentile between `0` and `100`.
///
/// # Returns
/// Returns the retained sample at the requested percentile, or `0` when the
/// input sample list is empty.
fn percentile_ns(samples: &[u64], percentile: usize) -> u64 {
  if samples.is_empty() {
    return 0;
  }

  let mut sorted_samples = samples.to_vec();
  sorted_samples.sort_unstable();

  let rank =
    ((percentile * sorted_samples.len()).saturating_add(99) / 100).max(1);
  let index = rank
    .saturating_sub(1)
    .min(sorted_samples.len().saturating_sub(1));

  return sorted_samples[index];
}

#[cfg(test)]
mod tests {
  use super::{
    build_scope_summary,
    CompletedFrame,
    SummaryReport,
    TraceReport,
    CPU_CLOCK_DOMAIN,
  };
  use crate::collector::cpu::{
    CpuEvent,
    CpuEventKind,
  };

  #[test]
  fn empty_scope_summary_uses_zeroed_aggregates() {
    let summary = build_scope_summary("tests.empty", CPU_CLOCK_DOMAIN, &[]);

    assert_eq!(summary.count(), 0);
    assert_eq!(summary.total_ns(), 0);
    assert_eq!(summary.mean_ns(), 0);
    assert_eq!(summary.p50_ns(), 0);
    assert_eq!(summary.p95_ns(), 0);
    assert_eq!(summary.max_ns(), 0);
  }

  #[test]
  fn single_sample_scope_summary_uses_the_only_sample_for_percentiles() {
    let summary = build_scope_summary("tests.single", CPU_CLOCK_DOMAIN, &[42]);

    assert_eq!(summary.count(), 1);
    assert_eq!(summary.total_ns(), 42);
    assert_eq!(summary.mean_ns(), 42);
    assert_eq!(summary.p50_ns(), 42);
    assert_eq!(summary.p95_ns(), 42);
    assert_eq!(summary.max_ns(), 42);
  }

  #[test]
  fn scope_summary_percentiles_use_nearest_rank() {
    let summary = build_scope_summary(
      "tests.percentiles",
      CPU_CLOCK_DOMAIN,
      &[10, 20, 30, 40, 50],
    );

    assert_eq!(summary.count(), 5);
    assert_eq!(summary.total_ns(), 150);
    assert_eq!(summary.mean_ns(), 30);
    assert_eq!(summary.p50_ns(), 30);
    assert_eq!(summary.p95_ns(), 50);
    assert_eq!(summary.max_ns(), 50);
  }

  #[test]
  fn completed_frame_summary_aggregates_scope_and_frame_metrics() {
    let thread_id = std::thread::current().id();
    let report = SummaryReport::from_completed_frames(
      "tests.report",
      "2026-04-07T00:00:00Z".to_owned(),
      &[
        CompletedFrame::cpu(
          0,
          100,
          vec![
            CpuEvent {
              name: "tests.scope",
              kind: CpuEventKind::SpanBegin,
              timestamp_ns: 10,
              thread_id,
            },
            CpuEvent {
              name: "tests.scope",
              kind: CpuEventKind::SpanEnd,
              timestamp_ns: 40,
              thread_id,
            },
          ],
        ),
        CompletedFrame::cpu(
          1,
          200,
          vec![
            CpuEvent {
              name: "tests.scope",
              kind: CpuEventKind::SpanBegin,
              timestamp_ns: 50,
              thread_id,
            },
            CpuEvent {
              name: "tests.scope",
              kind: CpuEventKind::SpanEnd,
              timestamp_ns: 110,
              thread_id,
            },
          ],
        ),
      ],
    );

    assert_eq!(report.run_name(), "tests.report");
    assert_eq!(report.clock_domains(), ["cpu"]);
    assert_eq!(report.frame_count(), 2);
    assert_eq!(report.frame_p50_ns(), 100);
    assert_eq!(report.frame_p95_ns(), 200);
    assert_eq!(report.frame_max_ns(), 200);
    assert_eq!(report.scopes().len(), 1);
    assert_eq!(report.scopes()[0].name(), "tests.scope");
    assert_eq!(report.scopes()[0].clock_domain(), "cpu");
    assert_eq!(report.scopes()[0].count(), 2);
    assert_eq!(report.scopes()[0].total_ns(), 90);
    assert_eq!(report.scopes()[0].mean_ns(), 45);
    assert_eq!(report.scopes()[0].p50_ns(), 30);
    assert_eq!(report.scopes()[0].p95_ns(), 60);
    assert_eq!(report.scopes()[0].max_ns(), 60);
  }

  #[test]
  fn trace_report_preserves_cpu_event_order_within_frames() {
    let thread_id = std::thread::current().id();
    let report = TraceReport::from_completed_frames(
      "tests.trace.order",
      "2026-04-07T00:00:00Z".to_owned(),
      &[CompletedFrame::cpu(
        3,
        100,
        vec![
          CpuEvent {
            name: "tests.scope",
            kind: CpuEventKind::SpanBegin,
            timestamp_ns: 10,
            thread_id,
          },
          CpuEvent {
            name: "tests.scope",
            kind: CpuEventKind::SpanEnd,
            timestamp_ns: 30,
            thread_id,
          },
        ],
      )],
    );

    assert_eq!(report.run_name(), "tests.trace.order");
    assert_eq!(report.generated_at_utc(), "2026-04-07T00:00:00Z");
    assert_eq!(report.events().len(), 2);
    assert_eq!(report.events()[0].kind(), "span_begin");
    assert_eq!(report.events()[1].kind(), "span_end");
    assert_eq!(report.events()[0].timestamp_ns(), 10);
    assert_eq!(report.events()[1].timestamp_ns(), 30);
    assert_eq!(report.events()[0].frame_index(), 3);
    assert_eq!(report.events()[1].frame_index(), 3);
    assert!(report.events()[0].thread_id().is_some());
  }

  #[test]
  fn trace_report_encodes_counter_and_instant_events() {
    let thread_id = std::thread::current().id();
    let report = TraceReport::from_completed_frames(
      "tests.trace.events",
      "2026-04-07T00:00:00Z".to_owned(),
      &[CompletedFrame::cpu(
        0,
        100,
        vec![
          CpuEvent {
            name: "tests.counter",
            kind: CpuEventKind::Counter { value: 9 },
            timestamp_ns: 11,
            thread_id,
          },
          CpuEvent {
            name: "tests.instant",
            kind: CpuEventKind::Instant,
            timestamp_ns: 12,
            thread_id,
          },
        ],
      )],
    );

    assert_eq!(report.events().len(), 2);
    assert_eq!(report.events()[0].name(), "tests.counter");
    assert_eq!(report.events()[0].kind(), "counter");
    assert_eq!(report.events()[0].value(), Some(9));
    assert_eq!(report.events()[1].name(), "tests.instant");
    assert_eq!(report.events()[1].kind(), "instant");
    assert_eq!(report.events()[1].value(), None);
  }

  #[cfg(feature = "export-summary")]
  #[test]
  fn summary_json_schema_fields_remain_stable() {
    use lambda_platform::profiling_support::deps::serde_json::Value;

    let report = SummaryReport::from_completed_frames(
      "tests.schema",
      "2026-04-07T00:00:00Z".to_owned(),
      &[CompletedFrame::cpu(0, 100, Vec::new())],
    );
    let json_value: Value =
      lambda_platform::profiling_support::deps::serde_json::from_slice(
        &report.to_json_bytes().unwrap(),
      )
      .unwrap();
    let root = json_value.as_object().unwrap();

    assert!(root.contains_key("schema_version"));
    assert!(root.contains_key("run_name"));
    assert!(root.contains_key("crate_version"));
    assert!(root.contains_key("clock_domains"));
    assert!(root.contains_key("generated_at_utc"));
    assert!(root.contains_key("scopes"));
    assert!(root.contains_key("frame_count"));
    assert!(root.contains_key("frame_p50_ns"));
    assert!(root.contains_key("frame_p95_ns"));
    assert!(root.contains_key("frame_max_ns"));
  }

  #[cfg(feature = "export-trace")]
  #[test]
  fn trace_json_schema_fields_remain_stable() {
    use lambda_platform::profiling_support::deps::serde_json::Value;

    let report = TraceReport::from_completed_frames(
      "tests.trace.schema",
      "2026-04-07T00:00:00Z".to_owned(),
      &[CompletedFrame::cpu(0, 100, Vec::new())],
    );
    let json_value: Value =
      lambda_platform::profiling_support::deps::serde_json::from_slice(
        &report.to_json_bytes().unwrap(),
      )
      .unwrap();
    let root = json_value.as_object().unwrap();

    assert!(root.contains_key("schema_version"));
    assert!(root.contains_key("run_name"));
    assert!(root.contains_key("generated_at_utc"));
    assert!(root.contains_key("events"));
  }

  #[cfg(feature = "export-trace")]
  #[test]
  fn trace_json_encodes_counter_and_instant_events() {
    use lambda_platform::profiling_support::deps::serde_json::Value;

    let thread_id = std::thread::current().id();
    let report = TraceReport::from_completed_frames(
      "tests.trace.json",
      "2026-04-07T00:00:00Z".to_owned(),
      &[CompletedFrame::cpu(
        4,
        100,
        vec![
          CpuEvent {
            name: "tests.counter",
            kind: CpuEventKind::Counter { value: 11 },
            timestamp_ns: 12,
            thread_id,
          },
          CpuEvent {
            name: "tests.instant",
            kind: CpuEventKind::Instant,
            timestamp_ns: 13,
            thread_id,
          },
        ],
      )],
    );
    let json_value: Value =
      lambda_platform::profiling_support::deps::serde_json::from_slice(
        &report.to_json_bytes().unwrap(),
      )
      .unwrap();
    let events = json_value
      .as_object()
      .unwrap()
      .get("events")
      .unwrap()
      .as_array()
      .unwrap();

    assert_eq!(events.len(), 2);
    assert_eq!(events[0].get("kind").unwrap(), "counter");
    assert_eq!(events[0].get("value").unwrap(), 11);
    assert_eq!(events[0].get("frame_index").unwrap(), 4);
    assert!(events[0].get("thread_id").is_some());
    assert_eq!(events[1].get("kind").unwrap(), "instant");
    assert!(events[1].get("value").is_none());
  }
}
