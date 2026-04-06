/// Aggregated profiling data for a completed run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SummaryReport {
  run_name: String,
  scope_count: usize,
}

impl SummaryReport {
  pub(crate) fn placeholder(run_name: &str) -> Self {
    return Self {
      run_name: run_name.to_owned(),
      scope_count: 0,
    };
  }
}

/// Ordered profiling events for a completed run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TraceReport {
  run_name: String,
  event_count: usize,
}

impl TraceReport {
  pub(crate) fn placeholder(run_name: &str) -> Self {
    return Self {
      run_name: run_name.to_owned(),
      event_count: 0,
    };
  }
}

/// Outcome of comparing a current profiling run against a baseline.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ComparisonReport {
  matched_scope_count: usize,
  violation_count: usize,
}
