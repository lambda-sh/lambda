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

/// Stable name used when reporting frame-level comparison results.
pub const FRAME_SCOPE_NAME: &str = "frame";

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

  /// Deserializes a summary report from stable JSON bytes.
  ///
  /// # Arguments
  /// - `json_bytes`: Stable summary JSON bytes produced by profiler export.
  ///
  /// # Returns
  /// Returns the deserialized summary report when parsing succeeds.
  ///
  /// # Errors
  /// Returns [`crate::CompareError::DeserializeSummaryReport`] when summary
  /// deserialization is unavailable or the input does not match the stable
  /// summary schema.
  pub fn from_json_bytes(
    json_bytes: &[u8],
  ) -> Result<Self, crate::CompareError> {
    #[cfg(feature = "export-summary")]
    {
      use lambda_platform::profiling_support::deps::serde_json::Value;

      let root: Value =
        lambda_platform::profiling_support::deps::serde_json::from_slice(
          json_bytes,
        )
        .map_err(|_| crate::CompareError::DeserializeSummaryReport)?;
      let root_object = root
        .as_object()
        .ok_or(crate::CompareError::DeserializeSummaryReport)?;
      let scopes_value = root_object
        .get("scopes")
        .ok_or(crate::CompareError::DeserializeSummaryReport)?;
      let scopes_array = scopes_value
        .as_array()
        .ok_or(crate::CompareError::DeserializeSummaryReport)?;
      let mut scopes = Vec::with_capacity(scopes_array.len());

      for scope_value in scopes_array {
        let scope_object = scope_value
          .as_object()
          .ok_or(crate::CompareError::DeserializeSummaryReport)?;

        scopes.push(SummaryScope {
          name: required_string_field(scope_object, "name")?,
          clock_domain: required_string_field(scope_object, "clock_domain")?,
          count: required_u64_field(scope_object, "count")?,
          total_ns: required_u64_field(scope_object, "total_ns")?,
          mean_ns: required_u64_field(scope_object, "mean_ns")?,
          p50_ns: required_u64_field(scope_object, "p50_ns")?,
          p95_ns: required_u64_field(scope_object, "p95_ns")?,
          max_ns: required_u64_field(scope_object, "max_ns")?,
        });
      }

      return Ok(Self {
        run_name: required_string_field(root_object, "run_name")?,
        crate_version: required_string_field(root_object, "crate_version")?,
        generated_at_utc: required_string_field(
          root_object,
          "generated_at_utc",
        )?,
        scopes,
        frame_count: required_u64_field(root_object, "frame_count")?,
        frame_p50_ns: required_u64_field(root_object, "frame_p50_ns")?,
        frame_p95_ns: required_u64_field(root_object, "frame_p95_ns")?,
        frame_max_ns: required_u64_field(root_object, "frame_max_ns")?,
      });
    }

    #[cfg(not(feature = "export-summary"))]
    {
      let _ = json_bytes;

      return Err(crate::CompareError::Unavailable);
    }
  }

  /// Returns the named scope or a comparison error when it is absent.
  ///
  /// This helper is primarily intended for deterministic regression fixtures
  /// that require a specific baseline scope to be present before comparison.
  ///
  /// # Arguments
  /// - `name`: Stable scope name to look up.
  /// - `clock_domain`: Stable clock-domain label to match.
  ///
  /// # Returns
  /// Returns the matching retained scope summary.
  ///
  /// # Errors
  /// Returns [`crate::CompareError::MissingBaselineScope`] when the requested
  /// scope is absent from the report.
  pub fn require_scope(
    &self,
    name: &str,
    clock_domain: &str,
  ) -> Result<&SummaryScope, crate::CompareError> {
    let scope = self
      .scopes()
      .iter()
      .find(|scope| {
        return scope.name() == name && scope.clock_domain() == clock_domain;
      })
      .ok_or_else(|| crate::CompareError::MissingBaselineScope {
        name: name.to_owned(),
        clock_domain: clock_domain.to_owned(),
      })?;

    return Ok(scope);
  }

  /// Compares this report against a baseline using explicit tolerances.
  ///
  /// # Arguments
  /// - `baseline`: Summary report treated as the regression baseline.
  /// - `tolerances`: Metric tolerances applied during comparison.
  ///
  /// # Returns
  /// Returns a comparison report describing matched scopes, violations, and
  /// informational extra current scopes.
  pub fn compare_against(
    &self,
    baseline: &Self,
    tolerances: &ComparisonTolerances,
  ) -> ComparisonReport {
    return ComparisonReport::compare(self, baseline, tolerances);
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
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComparisonMetric {
  /// Compares the arithmetic mean duration in nanoseconds.
  MeanNs,
  /// Compares the p95 duration in nanoseconds.
  P95Ns,
  /// Compares the maximum duration in nanoseconds.
  MaxNs,
}

impl ComparisonMetric {
  /// Returns the stable metric label used in comparison reporting.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the stable metric label.
  pub fn as_str(&self) -> &'static str {
    match self {
      Self::MeanNs => {
        return "mean_ns";
      }
      Self::P95Ns => {
        return "p95_ns";
      }
      Self::MaxNs => {
        return "max_ns";
      }
    }
  }
}

/// Stores one tolerance rule applied during summary comparison.
#[derive(Clone, Debug, PartialEq)]
pub struct MetricTolerance {
  metric: ComparisonMetric,
  relative_threshold: f64,
  absolute_threshold_ns: u64,
}

impl MetricTolerance {
  /// Creates a tolerance rule for one comparison metric.
  ///
  /// # Arguments
  /// - `metric`: Summary metric evaluated against the baseline.
  /// - `relative_threshold`: Fractional slowdown threshold, such as `0.10`
  ///   for ten percent.
  /// - `absolute_threshold_ns`: Absolute slowdown threshold in nanoseconds.
  ///
  /// # Returns
  /// Returns the configured metric tolerance.
  pub fn new(
    metric: ComparisonMetric,
    relative_threshold: f64,
    absolute_threshold_ns: u64,
  ) -> Self {
    return Self {
      metric,
      relative_threshold,
      absolute_threshold_ns,
    };
  }

  /// Returns the metric evaluated by this tolerance rule.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the configured comparison metric.
  pub fn metric(&self) -> ComparisonMetric {
    return self.metric;
  }

  /// Returns the fractional slowdown threshold for this rule.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the configured relative slowdown threshold.
  pub fn relative_threshold(&self) -> f64 {
    return self.relative_threshold;
  }

  /// Returns the absolute slowdown threshold for this rule.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the configured absolute slowdown threshold in nanoseconds.
  pub fn absolute_threshold_ns(&self) -> u64 {
    return self.absolute_threshold_ns;
  }
}

/// Groups frame-level and named-scope tolerances for summary comparison.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ComparisonTolerances {
  frame_metrics: Vec<MetricTolerance>,
  scope_metrics: Vec<MetricTolerance>,
}

impl ComparisonTolerances {
  /// Creates an empty comparison-tolerance set.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns an empty comparison-tolerance set.
  pub fn new() -> Self {
    return Self::default();
  }

  /// Creates the built-in CPU regression tolerances.
  ///
  /// The returned rules implement the specification defaults for CPU summary
  /// comparison: frame-level `p95_ns` and `max_ns`, plus inner-scope
  /// `mean_ns` and `p95_ns`, all using a `10%` relative slowdown threshold and
  /// a `250_000 ns` absolute slowdown threshold.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the built-in CPU tolerance set.
  pub fn cpu_defaults() -> Self {
    return Self::new()
      .with_frame_metric(MetricTolerance::new(
        ComparisonMetric::P95Ns,
        0.10,
        250_000,
      ))
      .with_frame_metric(MetricTolerance::new(
        ComparisonMetric::MaxNs,
        0.10,
        250_000,
      ))
      .with_scope_metric(MetricTolerance::new(
        ComparisonMetric::MeanNs,
        0.10,
        250_000,
      ))
      .with_scope_metric(MetricTolerance::new(
        ComparisonMetric::P95Ns,
        0.10,
        250_000,
      ));
  }

  /// Appends one frame-level tolerance rule.
  ///
  /// # Arguments
  /// - `tolerance`: Tolerance rule applied to synthesized frame metrics.
  ///
  /// # Returns
  /// Returns the updated tolerance set.
  pub fn with_frame_metric(mut self, tolerance: MetricTolerance) -> Self {
    self.frame_metrics.push(tolerance);

    return self;
  }

  /// Appends one named-scope tolerance rule.
  ///
  /// # Arguments
  /// - `tolerance`: Tolerance rule applied to named scope summaries.
  ///
  /// # Returns
  /// Returns the updated tolerance set.
  pub fn with_scope_metric(mut self, tolerance: MetricTolerance) -> Self {
    self.scope_metrics.push(tolerance);

    return self;
  }

  /// Returns the configured frame-level tolerance rules.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the frame-level tolerance slice.
  pub fn frame_metrics(&self) -> &[MetricTolerance] {
    return &self.frame_metrics;
  }

  /// Returns the configured named-scope tolerance rules.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the named-scope tolerance slice.
  pub fn scope_metrics(&self) -> &[MetricTolerance] {
    return &self.scope_metrics;
  }
}

/// Identifies one scope encountered during summary comparison.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComparisonScope {
  name: String,
  clock_domain: String,
}

impl ComparisonScope {
  /// Returns the stable scope name.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the retained comparison scope name.
  pub fn name(&self) -> &str {
    return &self.name;
  }

  /// Returns the stable clock-domain label.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the retained comparison scope clock domain.
  pub fn clock_domain(&self) -> &str {
    return &self.clock_domain;
  }
}

/// Records one metric regression found during summary comparison.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComparisonViolation {
  scope: ComparisonScope,
  metric: ComparisonMetric,
  baseline_ns: u64,
  current_ns: u64,
  absolute_regression_ns: u64,
}

impl ComparisonViolation {
  /// Returns the scope associated with this violation.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the violating comparison scope.
  pub fn scope(&self) -> &ComparisonScope {
    return &self.scope;
  }

  /// Returns the metric that violated the configured tolerance.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the violating comparison metric.
  pub fn metric(&self) -> ComparisonMetric {
    return self.metric;
  }

  /// Returns the baseline metric value in nanoseconds.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the baseline metric value in nanoseconds.
  pub fn baseline_ns(&self) -> u64 {
    return self.baseline_ns;
  }

  /// Returns the current metric value in nanoseconds.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the current metric value in nanoseconds.
  pub fn current_ns(&self) -> u64 {
    return self.current_ns;
  }

  /// Returns the absolute regression size in nanoseconds.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the current-minus-baseline regression size in nanoseconds.
  pub fn absolute_regression_ns(&self) -> u64 {
    return self.absolute_regression_ns;
  }
}

/// Outcome of comparing a current profiling run against a baseline.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ComparisonReport {
  matched_scope_count: usize,
  violations: Vec<ComparisonViolation>,
  missing_current_scopes: Vec<ComparisonScope>,
  extra_current_scopes: Vec<ComparisonScope>,
}

impl ComparisonReport {
  /// Compares a current summary report against a baseline.
  ///
  /// # Arguments
  /// - `current`: Summary report produced by the current run.
  /// - `baseline`: Summary report used as the baseline.
  /// - `tolerances`: Metric tolerances applied during comparison.
  ///
  /// # Returns
  /// Returns the comparison outcome, including violations and informational
  /// extra current scopes.
  pub fn compare(
    current: &SummaryReport,
    baseline: &SummaryReport,
    tolerances: &ComparisonTolerances,
  ) -> Self {
    let mut matched_scope_count = 0;
    let mut violations = Vec::new();
    let mut missing_current_scopes = Vec::new();
    let mut extra_current_scopes = Vec::new();
    let baseline_scopes = compareable_scopes(baseline);
    let current_scopes = compareable_scopes(current);

    for (scope_key, baseline_values) in &baseline_scopes {
      let Some(current_values) = current_scopes.get(scope_key) else {
        missing_current_scopes.push(ComparisonScope {
          name: scope_key.0.clone(),
          clock_domain: scope_key.1.clone(),
        });
        continue;
      };

      matched_scope_count += 1;
      let applicable_tolerances = if scope_key.0 == FRAME_SCOPE_NAME {
        tolerances.frame_metrics()
      } else {
        tolerances.scope_metrics()
      };

      for tolerance in applicable_tolerances {
        let baseline_metric =
          metric_value_for_scope(baseline_values, tolerance.metric());
        let current_metric =
          metric_value_for_scope(current_values, tolerance.metric());

        if let Some(violation) = comparison_violation(
          scope_key,
          tolerance,
          baseline_metric,
          current_metric,
        ) {
          violations.push(violation);
        }
      }
    }

    for scope_key in current_scopes.keys() {
      if baseline_scopes.contains_key(scope_key) {
        continue;
      }

      extra_current_scopes.push(ComparisonScope {
        name: scope_key.0.clone(),
        clock_domain: scope_key.1.clone(),
      });
    }

    extra_current_scopes.sort_by(|left, right| {
      return left
        .clock_domain()
        .cmp(right.clock_domain())
        .then_with(|| left.name().cmp(right.name()));
    });

    return Self {
      matched_scope_count,
      violations,
      missing_current_scopes,
      extra_current_scopes,
    };
  }

  /// Returns whether the comparison passed.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns `true` when no metric violations or missing current scopes were
  /// recorded.
  pub fn passed(&self) -> bool {
    return self.violations.is_empty()
      && self.missing_current_scopes.is_empty();
  }

  /// Returns the number of scopes matched against the baseline.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the number of scopes present in both reports.
  pub fn matched_scope_count(&self) -> usize {
    return self.matched_scope_count;
  }

  /// Returns the recorded metric violations.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the ordered violation slice.
  pub fn violations(&self) -> &[ComparisonViolation] {
    return &self.violations;
  }

  /// Returns the baseline scopes absent from the current run.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the ordered missing-current scope slice.
  pub fn missing_current_scopes(&self) -> &[ComparisonScope] {
    return &self.missing_current_scopes;
  }

  /// Returns the current scopes absent from the baseline.
  ///
  /// These entries are informational and do not fail the comparison by
  /// themselves.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the ordered extra-current scope slice.
  pub fn extra_current_scopes(&self) -> &[ComparisonScope] {
    return &self.extra_current_scopes;
  }

  /// Returns the number of recorded metric violations.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the number of recorded metric violations.
  pub fn violation_count(&self) -> usize {
    return self.violations.len();
  }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ScopeMetricValues {
  mean_ns: u64,
  p95_ns: u64,
  max_ns: u64,
}

/// Returns one required string field from a summary JSON object.
#[cfg(feature = "export-summary")]
fn required_string_field(
  object: &lambda_platform::profiling_support::deps::serde_json::Map<
    String,
    lambda_platform::profiling_support::deps::serde_json::Value,
  >,
  field_name: &str,
) -> Result<String, crate::CompareError> {
  let value = object
    .get(field_name)
    .and_then(|value| value.as_str())
    .ok_or(crate::CompareError::DeserializeSummaryReport)?;

  return Ok(value.to_owned());
}

/// Returns one required unsigned integer field from a summary JSON object.
#[cfg(feature = "export-summary")]
fn required_u64_field(
  object: &lambda_platform::profiling_support::deps::serde_json::Map<
    String,
    lambda_platform::profiling_support::deps::serde_json::Value,
  >,
  field_name: &str,
) -> Result<u64, crate::CompareError> {
  return object
    .get(field_name)
    .and_then(|value| value.as_u64())
    .ok_or(crate::CompareError::DeserializeSummaryReport);
}

/// Builds the stable comparison map for one summary report.
fn compareable_scopes(
  report: &SummaryReport,
) -> BTreeMap<(String, String), ScopeMetricValues> {
  let mut scopes = BTreeMap::new();

  scopes.insert(
    (FRAME_SCOPE_NAME.to_owned(), CPU_CLOCK_DOMAIN.to_owned()),
    ScopeMetricValues {
      mean_ns: 0,
      p95_ns: report.frame_p95_ns(),
      max_ns: report.frame_max_ns(),
    },
  );

  for scope in report.scopes() {
    scopes.insert(
      (scope.name().to_owned(), scope.clock_domain().to_owned()),
      ScopeMetricValues {
        mean_ns: scope.mean_ns(),
        p95_ns: scope.p95_ns(),
        max_ns: scope.max_ns(),
      },
    );
  }

  return scopes;
}

/// Returns the requested metric value from one comparison scope record.
fn metric_value_for_scope(
  metric_values: &ScopeMetricValues,
  metric: ComparisonMetric,
) -> u64 {
  match metric {
    ComparisonMetric::MeanNs => {
      return metric_values.mean_ns;
    }
    ComparisonMetric::P95Ns => {
      return metric_values.p95_ns;
    }
    ComparisonMetric::MaxNs => {
      return metric_values.max_ns;
    }
  }
}

/// Evaluates one tolerance rule for one matched comparison scope.
fn comparison_violation(
  scope_key: &(String, String),
  tolerance: &MetricTolerance,
  baseline_ns: u64,
  current_ns: u64,
) -> Option<ComparisonViolation> {
  if current_ns <= baseline_ns {
    return None;
  }

  let absolute_regression_ns = current_ns.saturating_sub(baseline_ns);
  let relative_regression = if baseline_ns == 0 {
    f64::INFINITY
  } else {
    absolute_regression_ns as f64 / baseline_ns as f64
  };

  if absolute_regression_ns < tolerance.absolute_threshold_ns()
    || relative_regression <= tolerance.relative_threshold()
  {
    return None;
  }

  return Some(ComparisonViolation {
    scope: ComparisonScope {
      name: scope_key.0.clone(),
      clock_domain: scope_key.1.clone(),
    },
    metric: tolerance.metric(),
    baseline_ns,
    current_ns,
    absolute_regression_ns,
  });
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
    ComparisonMetric,
    ComparisonTolerances,
    CompletedFrame,
    MetricTolerance,
    SummaryReport,
    TraceReport,
    CPU_CLOCK_DOMAIN,
    FRAME_SCOPE_NAME,
  };
  use crate::collector::cpu::{
    CpuEvent,
    CpuEventKind,
  };

  fn summary_report_with_scope(
    run_name: &str,
    scope_name: &str,
    frame_p95_ns: u64,
    frame_max_ns: u64,
    scope_mean_ns: u64,
    scope_p95_ns: u64,
    scope_max_ns: u64,
  ) -> SummaryReport {
    return SummaryReport {
      run_name: run_name.to_owned(),
      crate_version: env!("CARGO_PKG_VERSION").to_owned(),
      generated_at_utc: "2026-04-08T00:00:00Z".to_owned(),
      scopes: vec![super::SummaryScope {
        name: scope_name.to_owned(),
        clock_domain: CPU_CLOCK_DOMAIN.to_owned(),
        count: 4,
        total_ns: scope_mean_ns.saturating_mul(4),
        mean_ns: scope_mean_ns,
        p50_ns: scope_mean_ns,
        p95_ns: scope_p95_ns,
        max_ns: scope_max_ns,
      }],
      frame_count: 4,
      frame_p50_ns: frame_p95_ns,
      frame_p95_ns,
      frame_max_ns,
    };
  }

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

  #[test]
  fn compare_uses_relative_threshold_only_when_absolute_threshold_is_zero() {
    let baseline = summary_report_with_scope(
      "tests.compare.relative.baseline",
      "tests.scope",
      1_000,
      1_000,
      1_000,
      1_000,
      1_000,
    );
    let current = summary_report_with_scope(
      "tests.compare.relative.current",
      "tests.scope",
      1_050,
      1_050,
      1_200,
      1_200,
      1_200,
    );
    let tolerances = ComparisonTolerances::new().with_scope_metric(
      MetricTolerance::new(ComparisonMetric::MeanNs, 0.10, 0),
    );
    let comparison = current.compare_against(&baseline, &tolerances);

    assert!(!comparison.passed());
    assert_eq!(comparison.violation_count(), 1);
    assert_eq!(comparison.violations()[0].scope().name(), "tests.scope");
    assert_eq!(
      comparison.violations()[0].scope().clock_domain(),
      CPU_CLOCK_DOMAIN
    );
    assert_eq!(
      comparison.violations()[0].metric(),
      ComparisonMetric::MeanNs
    );
  }

  #[test]
  fn compare_uses_absolute_threshold_only_when_relative_threshold_is_zero() {
    let baseline = summary_report_with_scope(
      "tests.compare.absolute.baseline",
      "tests.scope",
      1_000,
      1_000,
      1_000,
      1_000,
      1_000,
    );
    let current = summary_report_with_scope(
      "tests.compare.absolute.current",
      "tests.scope",
      1_001,
      1_001,
      1_050,
      1_050,
      1_050,
    );
    let tolerances = ComparisonTolerances::new().with_scope_metric(
      MetricTolerance::new(ComparisonMetric::MeanNs, 0.0, 40),
    );
    let comparison = current.compare_against(&baseline, &tolerances);

    assert!(!comparison.passed());
    assert_eq!(comparison.violation_count(), 1);
    assert_eq!(comparison.violations()[0].absolute_regression_ns(), 50);
  }

  #[test]
  fn compare_requires_both_relative_and_absolute_thresholds_to_fail() {
    let baseline = summary_report_with_scope(
      "tests.compare.combined.baseline",
      "tests.scope",
      1_000,
      1_000,
      1_000,
      1_000,
      1_000,
    );
    let current = summary_report_with_scope(
      "tests.compare.combined.current",
      "tests.scope",
      1_001,
      1_001,
      1_050,
      1_050,
      1_050,
    );
    let tolerances = ComparisonTolerances::new().with_scope_metric(
      MetricTolerance::new(ComparisonMetric::MeanNs, 0.10, 100),
    );
    let comparison = current.compare_against(&baseline, &tolerances);

    assert!(comparison.passed());
    assert_eq!(comparison.violation_count(), 0);
  }

  #[test]
  fn require_scope_reports_missing_baseline_scope() {
    let baseline = summary_report_with_scope(
      "tests.compare.missing.baseline",
      "tests.scope",
      1_000,
      1_000,
      1_000,
      1_000,
      1_000,
    );
    let error = baseline
      .require_scope("tests.missing", CPU_CLOCK_DOMAIN)
      .unwrap_err();

    assert_eq!(
      error,
      crate::CompareError::MissingBaselineScope {
        name: "tests.missing".to_owned(),
        clock_domain: CPU_CLOCK_DOMAIN.to_owned(),
      }
    );
  }

  #[test]
  fn compare_reports_extra_current_scope_as_informational() {
    let baseline = summary_report_with_scope(
      "tests.compare.extra.baseline",
      "tests.baseline",
      1_000,
      1_000,
      1_000,
      1_000,
      1_000,
    );
    let current = SummaryReport {
      run_name: "tests.compare.extra.current".to_owned(),
      crate_version: env!("CARGO_PKG_VERSION").to_owned(),
      generated_at_utc: "2026-04-08T00:00:00Z".to_owned(),
      scopes: vec![
        super::SummaryScope {
          name: "tests.baseline".to_owned(),
          clock_domain: CPU_CLOCK_DOMAIN.to_owned(),
          count: 4,
          total_ns: 4_000,
          mean_ns: 1_000,
          p50_ns: 1_000,
          p95_ns: 1_000,
          max_ns: 1_000,
        },
        super::SummaryScope {
          name: "tests.extra".to_owned(),
          clock_domain: CPU_CLOCK_DOMAIN.to_owned(),
          count: 4,
          total_ns: 4_000,
          mean_ns: 1_000,
          p50_ns: 1_000,
          p95_ns: 1_000,
          max_ns: 1_000,
        },
      ],
      frame_count: 4,
      frame_p50_ns: 1_000,
      frame_p95_ns: 1_000,
      frame_max_ns: 1_000,
    };
    let comparison =
      current.compare_against(&baseline, &ComparisonTolerances::cpu_defaults());

    assert!(comparison.passed());
    assert_eq!(comparison.matched_scope_count(), 2);
    assert_eq!(comparison.extra_current_scopes().len(), 1);
    assert_eq!(comparison.extra_current_scopes()[0].name(), "tests.extra");
    assert_eq!(
      comparison.extra_current_scopes()[0].clock_domain(),
      CPU_CLOCK_DOMAIN
    );
    assert_eq!(comparison.missing_current_scopes().len(), 0);
    assert_eq!(FRAME_SCOPE_NAME, "frame");
  }
}
