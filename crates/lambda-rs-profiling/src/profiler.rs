use std::{
  path::PathBuf,
  sync::{
    Arc,
    OnceLock,
    RwLock,
  },
  time::Instant,
};

use crate::{
  collector::cpu::CpuCollector,
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
}

static GLOBAL_PROFILER: OnceLock<RwLock<Option<&'static Profiler>>> =
  OnceLock::new();

fn global_profiler_slot() -> &'static RwLock<Option<&'static Profiler>> {
  return GLOBAL_PROFILER.get_or_init(|| RwLock::new(None));
}

impl Profiler {
  pub(crate) fn from_builder(builder: ProfilerBuilder) -> Self {
    return Self {
      run_name: builder.run_name().to_owned(),
      summary_export_enabled: builder.summary_export_enabled(),
      trace_export_enabled: builder.trace_export_enabled(),
      output_directory: builder.output_directory().map(PathBuf::from),
      gpu_timestamps_enabled: builder.gpu_timestamps_enabled(),
      frame_history_limit: builder.frame_history_limit(),
      cpu_collector: CpuCollector::new(),
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
  /// Returns [`ProfilerFlushError`] when frame flushing is not yet available.
  pub fn flush_frame(&self) -> Result<FrameArtifacts, ProfilerFlushError> {
    self.cpu_collector.flush_thread_local_buffers();

    let _ = (
      &self.output_directory,
      self.gpu_timestamps_enabled,
      self.frame_history_limit,
    );

    return Ok(FrameArtifacts::default());
  }

  /// Exports an aggregated summary report.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns a placeholder summary report for the current profiler
  /// configuration.
  ///
  /// # Errors
  /// Returns [`ExportError`] when summary export is disabled or unavailable.
  pub fn export_summary(&self) -> Result<SummaryReport, ExportError> {
    if !self.summary_export_enabled {
      return Err(ExportError::Disabled {
        export_name: "summary",
      });
    }

    return Ok(SummaryReport::placeholder(&self.run_name));
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
