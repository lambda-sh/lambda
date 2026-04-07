use std::{
  collections::VecDeque,
  fs,
  path::PathBuf,
  sync::{
    Arc,
    Mutex,
    OnceLock,
    RwLock,
  },
  time::Instant,
};

use crate::{
  collector::cpu::CpuCollector,
  report::CompletedFrame,
  ExportError,
  ProfilerBuilder,
  ProfilerFlushError,
  ProfilerInstallError,
  SummaryReport,
  TraceReport,
};

/// Records and exports profiling data for a named run.
#[derive(Clone, Debug)]
pub struct Profiler {
  run_name: String,
  summary_export_enabled: bool,
  trace_export_enabled: bool,
  output_directory: Option<PathBuf>,
  gpu_timestamps_enabled: bool,
  frame_history_limit: usize,
  cpu_collector: Arc<CpuCollector>,
  state: Arc<Mutex<ProfilerState>>,
}

static GLOBAL_PROFILER: OnceLock<RwLock<Option<&'static Profiler>>> =
  OnceLock::new();

fn global_profiler_slot() -> &'static RwLock<Option<&'static Profiler>> {
  return GLOBAL_PROFILER.get_or_init(|| RwLock::new(None));
}

impl Profiler {
  pub(crate) fn from_builder(builder: ProfilerBuilder) -> Self {
    let cpu_collector = CpuCollector::new();

    return Self {
      run_name: builder.run_name().to_owned(),
      summary_export_enabled: builder.summary_export_enabled(),
      trace_export_enabled: builder.trace_export_enabled(),
      output_directory: builder.output_directory().map(PathBuf::from),
      gpu_timestamps_enabled: builder.gpu_timestamps_enabled(),
      frame_history_limit: builder.frame_history_limit(),
      state: Arc::new(Mutex::new(ProfilerState::new(
        builder.frame_history_limit(),
        cpu_collector.current_timestamp_ns(),
      ))),
      cpu_collector,
    };
  }

  /// Installs this profiler as the process-wide global profiler.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns `Ok(())` when global installation succeeds.
  ///
  /// # Errors
  /// Returns [`ProfilerInstallError::AlreadyInstalled`] when another profiler
  /// has already been installed globally.
  pub fn install_global(self) -> Result<(), ProfilerInstallError> {
    let mut profiler_slot = global_profiler_slot()
      .write()
      .unwrap_or_else(|poisoned| poisoned.into_inner());

    if profiler_slot.is_some() {
      return Err(ProfilerInstallError::AlreadyInstalled);
    }

    *profiler_slot = Some(Box::leak(Box::new(self)));

    return Ok(());
  }

  /// Returns the globally installed profiler when one is available.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the current global profiler, or `None` when no global profiler is
  /// installed.
  pub fn global() -> Option<&'static Profiler> {
    let profiler_slot = global_profiler_slot()
      .read()
      .unwrap_or_else(|poisoned| poisoned.into_inner());

    return *profiler_slot;
  }

  /// Begins a named profiling scope.
  ///
  /// # Arguments
  /// - `name`: Stable name recorded for the scope.
  ///
  /// # Returns
  /// Returns a guard that closes the scope when it is dropped.
  pub fn begin_scope(&self, name: &'static str) -> ScopeGuard {
    let started_at = Instant::now();

    self.cpu_collector.record_span_begin(name, started_at);

    return ScopeGuard::enabled(
      name,
      Arc::clone(&self.cpu_collector),
      started_at,
    );
  }

  /// Records a counter value for the current frame or scope.
  ///
  /// # Arguments
  /// - `name`: Stable counter name.
  /// - `value`: Counter value recorded by the caller.
  ///
  /// # Returns
  /// This method does not return a value.
  pub fn record_counter(&self, name: &'static str, value: u64) {
    self.cpu_collector.record_counter(name, value);
  }

  /// Records an instantaneous profiling event.
  ///
  /// # Arguments
  /// - `name`: Stable instant event name.
  ///
  /// # Returns
  /// This method does not return a value.
  pub fn record_instant(&self, name: &'static str) {
    self.cpu_collector.record_instant(name);
  }

  /// Flushes profiler data for the current frame.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns frame artifact metadata for the completed flush.
  ///
  /// # Errors
  /// Returns [`ProfilerFlushError`] when summary export is requested but the
  /// artifact cannot be generated or written.
  pub fn flush_frame(&self) -> Result<FrameArtifacts, ProfilerFlushError> {
    let frame_end_ns = self.cpu_collector.current_timestamp_ns();
    let cpu_events = self.cpu_collector.take_events();
    let completed_frame = {
      let mut state = self
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
      let completed_frame = state.complete_frame(frame_end_ns, cpu_events);

      state.retain_completed_frame(completed_frame.clone());

      completed_frame
    };

    let _ = (
      self.gpu_timestamps_enabled,
      self.frame_history_limit,
      completed_frame.frame_index(),
    );

    let mut frame_artifacts = FrameArtifacts::default();

    if self.summary_export_enabled {
      let summary_report =
        self.export_summary().map_err(ProfilerFlushError::Export)?;
      let summary_path = self
        .write_summary_report(&summary_report)
        .map_err(ProfilerFlushError::Export)?;
      frame_artifacts.summary_path = Some(summary_path);
    }

    return Ok(frame_artifacts);
  }

  /// Exports an aggregated summary report.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns an aggregated summary report over the retained completed frames.
  ///
  /// # Errors
  /// Returns [`ExportError`] when summary export is disabled or unavailable.
  pub fn export_summary(&self) -> Result<SummaryReport, ExportError> {
    if !self.summary_export_enabled {
      return Err(ExportError::Disabled {
        export_name: "summary",
      });
    }

    let generated_at_utc = summary_generated_at_utc()?;
    let completed_frames = self.completed_frames();

    return Ok(SummaryReport::from_completed_frames(
      &self.run_name,
      generated_at_utc,
      &completed_frames,
    ));
  }

  /// Exports an ordered trace report.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns a placeholder trace report for the current profiler
  /// configuration.
  ///
  /// # Errors
  /// Returns [`ExportError`] when trace export is disabled or unavailable.
  pub fn export_trace(&self) -> Result<TraceReport, ExportError> {
    if !self.trace_export_enabled {
      return Err(ExportError::Disabled {
        export_name: "trace",
      });
    }

    return Ok(TraceReport::placeholder(&self.run_name));
  }

  #[cfg(test)]
  pub(crate) fn reset_global_for_tests() {
    let mut profiler_slot = global_profiler_slot()
      .write()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    *profiler_slot = None;
  }

  /// Returns the retained completed frame history.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns the retained completed frame history used for export.
  fn completed_frames(&self) -> Vec<CompletedFrame> {
    let state = self
      .state
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());

    return state.completed_frames.iter().cloned().collect();
  }

  /// Writes a summary report into the configured output directory.
  ///
  /// # Arguments
  /// - `summary_report`: Summary report to serialize and write.
  ///
  /// # Returns
  /// Returns the path written for the summary artifact.
  ///
  /// # Errors
  /// Returns [`ExportError`] when the output directory is unavailable or the
  /// artifact cannot be serialized or written.
  fn write_summary_report(
    &self,
    summary_report: &SummaryReport,
  ) -> Result<PathBuf, ExportError> {
    let Some(output_directory) = &self.output_directory else {
      return Err(ExportError::InvalidOutputDirectory {
        path: PathBuf::from("<missing>"),
      });
    };
    let summary_path = output_directory.join(format!(
      "{}-summary.json",
      sanitize_run_name_for_path(&self.run_name)
    ));
    let json_bytes = summary_report.to_json_bytes()?;

    fs::write(&summary_path, json_bytes).map_err(|_| {
      return ExportError::WriteFailed {
        path: summary_path.clone(),
      };
    })?;

    return Ok(summary_path);
  }
}

#[derive(Clone, Debug)]
struct ProfilerState {
  completed_frames: VecDeque<CompletedFrame>,
  next_frame_index: usize,
  last_frame_boundary_ns: u64,
  frame_history_limit: usize,
}

impl ProfilerState {
  /// Creates retained state for a profiler instance.
  ///
  /// # Arguments
  /// - `frame_history_limit`: Maximum number of completed frames to retain.
  /// - `last_frame_boundary_ns`: Initial collector-relative frame boundary.
  ///
  /// # Returns
  /// Returns empty retained state for a profiler instance.
  fn new(frame_history_limit: usize, last_frame_boundary_ns: u64) -> Self {
    return Self {
      completed_frames: VecDeque::with_capacity(frame_history_limit),
      next_frame_index: 0,
      last_frame_boundary_ns,
      frame_history_limit,
    };
  }

  /// Completes the active frame and returns the retained frame record.
  ///
  /// # Arguments
  /// - `frame_end_ns`: Collector-relative frame end timestamp.
  /// - `cpu_events`: CPU events drained for the completed frame.
  ///
  /// # Returns
  /// Returns the completed frame record for the current flush.
  fn complete_frame(
    &mut self,
    frame_end_ns: u64,
    cpu_events: Vec<crate::collector::cpu::CpuEvent>,
  ) -> CompletedFrame {
    let frame_index = self.next_frame_index;
    let frame_duration_ns =
      frame_end_ns.saturating_sub(self.last_frame_boundary_ns);

    self.next_frame_index += 1;
    self.last_frame_boundary_ns = frame_end_ns;

    return CompletedFrame::cpu(frame_index, frame_duration_ns, cpu_events);
  }

  /// Retains a completed frame while honoring the history limit.
  ///
  /// # Arguments
  /// - `completed_frame`: Completed frame to retain.
  ///
  /// # Returns
  /// This method does not return a value.
  fn retain_completed_frame(&mut self, completed_frame: CompletedFrame) {
    self.completed_frames.push_back(completed_frame);

    while self.completed_frames.len() > self.frame_history_limit {
      self.completed_frames.pop_front();
    }
  }
}

/// Returns the current UTC timestamp used for summary export.
///
/// # Arguments
/// This function does not take additional arguments.
///
/// # Returns
/// Returns the formatted UTC timestamp used in summary exports.
///
/// # Errors
/// Returns [`ExportError::Serialize`] when timestamp formatting is
/// unavailable.
fn summary_generated_at_utc() -> Result<String, ExportError> {
  #[cfg(feature = "export-summary")]
  {
    use lambda_platform::profiling_support::deps::time::{
      format_description::well_known::Rfc3339,
      OffsetDateTime,
    };

    let now_utc = OffsetDateTime::now_utc();

    return now_utc.format(&Rfc3339).map_err(|_| {
      return ExportError::Serialize {
        export_name: "summary",
      };
    });
  }

  #[cfg(not(feature = "export-summary"))]
  {
    return Err(ExportError::Unavailable {
      export_name: "summary",
    });
  }
}

/// Converts a run name into a filesystem-safe artifact stem.
///
/// # Arguments
/// - `run_name`: Configured profiler run name.
///
/// # Returns
/// Returns a filesystem-safe artifact stem derived from the run name.
fn sanitize_run_name_for_path(run_name: &str) -> String {
  let sanitized_name: String = run_name
    .chars()
    .map(|character| {
      if character.is_ascii_alphanumeric()
        || character == '-'
        || character == '_'
      {
        return character;
      }

      return '-';
    })
    .collect();

  if sanitized_name.is_empty() {
    return String::from("profiling");
  }

  return sanitized_name;
}

/// Tracks a profiling scope and closes it when dropped.
#[derive(Debug)]
pub struct ScopeGuard {
  #[allow(dead_code)]
  name: &'static str,
  closed: bool,
  collector: Option<Arc<CpuCollector>>,
  started_at: Option<Instant>,
}

impl ScopeGuard {
  pub(crate) fn disabled(name: &'static str) -> Self {
    return Self {
      name,
      closed: false,
      collector: None,
      started_at: None,
    };
  }

  pub(crate) fn enabled(
    name: &'static str,
    collector: Arc<CpuCollector>,
    started_at: Instant,
  ) -> Self {
    return Self {
      name,
      closed: false,
      collector: Some(collector),
      started_at: Some(started_at),
    };
  }

  #[allow(dead_code)]
  #[cfg(test)]
  pub(crate) fn name(&self) -> &'static str {
    return self.name;
  }

  #[allow(dead_code)]
  #[cfg(test)]
  pub(crate) fn is_closed(&self) -> bool {
    return self.closed;
  }

  pub(crate) fn close(&mut self) {
    if self.closed {
      return;
    }

    if let Some(collector) = self.collector.take() {
      let ended_at = Instant::now();
      let _ = self.started_at.take();

      collector.record_span_end(self.name, ended_at);
    }

    self.closed = true;
  }
}

impl Drop for ScopeGuard {
  fn drop(&mut self) {
    self.close();
  }
}

/// Describes the files written during a frame flush.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FrameArtifacts {
  /// Path to the written summary artifact, when summary export ran.
  pub summary_path: Option<PathBuf>,
  /// Path to the written trace artifact, when trace export ran.
  pub trace_path: Option<PathBuf>,
}

#[cfg(test)]
mod install_tests {
  use std::sync::{
    Mutex,
    MutexGuard,
    OnceLock,
  };

  use super::Profiler;
  use crate::{
    ProfilerBuilder,
    ProfilerInstallError,
  };

  static INSTALL_TEST_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();

  fn install_test_mutex() -> &'static Mutex<()> {
    return INSTALL_TEST_MUTEX.get_or_init(|| Mutex::new(()));
  }

  struct InstallTestContext {
    _guard: MutexGuard<'static, ()>,
  }

  impl InstallTestContext {
    fn new() -> Self {
      let guard = install_test_mutex()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
      Profiler::reset_global_for_tests();

      return Self { _guard: guard };
    }
  }

  impl Drop for InstallTestContext {
    fn drop(&mut self) {
      Profiler::reset_global_for_tests();
    }
  }

  #[test]
  fn install_global_returns_already_installed_for_second_profiler() {
    let _context = InstallTestContext::new();
    let first_profiler = ProfilerBuilder::new("tests.install.first_profiler")
      .build()
      .unwrap();
    let second_profiler = ProfilerBuilder::new("tests.install.second_profiler")
      .build()
      .unwrap();

    first_profiler.install_global().unwrap();

    let error = second_profiler.install_global().unwrap_err();

    assert_eq!(error, ProfilerInstallError::AlreadyInstalled);
  }

  #[test]
  fn install_global_makes_profiler_visible_via_global() {
    let _context = InstallTestContext::new();
    let profiler = ProfilerBuilder::new("tests.install.global_visibility")
      .build()
      .unwrap();

    profiler.install_global().unwrap();

    let global_profiler = Profiler::global().unwrap();
    let guard = global_profiler.begin_scope("tests.install.scope");

    assert_eq!(guard.name(), "tests.install.scope");
  }
}

#[cfg(test)]
mod flush_tests {
  #[cfg(feature = "export-summary")]
  use std::{
    env,
    fs,
    path::PathBuf,
    process,
    sync::atomic::{
      AtomicU64,
      Ordering,
    },
  };
  use std::{
    thread,
    time::Duration,
  };

  use crate::ProfilerBuilder;

  #[cfg(feature = "export-summary")]
  static NEXT_TEST_ID: AtomicU64 = AtomicU64::new(0);

  #[cfg(feature = "export-summary")]
  fn unique_test_path(suffix: &str) -> PathBuf {
    let test_id = NEXT_TEST_ID.fetch_add(1, Ordering::Relaxed);

    return env::temp_dir().join(format!(
      "lambda-rs-profiling-{suffix}-{}-{test_id}",
      process::id()
    ));
  }

  #[cfg(feature = "export-summary")]
  fn remove_path_if_present(path: &PathBuf) {
    if !path.exists() {
      return;
    }

    if path.is_dir() {
      let _ = fs::remove_dir_all(path);
      return;
    }

    let _ = fs::remove_file(path);
  }

  #[test]
  fn flush_frame_returns_empty_artifacts_when_summary_export_is_disabled() {
    let profiler = ProfilerBuilder::new("tests.flush.no_export")
      .build()
      .unwrap();

    {
      let _guard = profiler.begin_scope("tests.flush.scope");
      thread::sleep(Duration::from_millis(1));
    }

    let artifacts = profiler.flush_frame().unwrap();

    assert_eq!(artifacts.summary_path, None);
    assert_eq!(artifacts.trace_path, None);
  }

  #[cfg(feature = "export-summary")]
  #[test]
  fn flush_frame_writes_summary_artifact_when_summary_export_is_enabled() {
    let output_directory = unique_test_path("summary-export");
    remove_path_if_present(&output_directory);

    let profiler = ProfilerBuilder::new("tests.summary export")
      .with_summary_export(true)
      .with_output_directory(&output_directory)
      .build()
      .unwrap();

    {
      let _guard = profiler.begin_scope("tests.flush.scope");
      thread::sleep(Duration::from_millis(1));
    }

    let artifacts = profiler.flush_frame().unwrap();
    let summary_path = artifacts.summary_path.unwrap();
    let summary_contents = fs::read_to_string(&summary_path).unwrap();

    assert!(summary_path.exists());
    assert!(summary_contents.contains("\"schema_version\""));
    assert!(summary_contents.contains("\"frame_count\""));
    assert!(summary_contents.contains("\"scopes\""));

    remove_path_if_present(&output_directory);
  }
}
