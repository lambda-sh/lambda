use std::path::{
  Path,
  PathBuf,
};

use crate::{
  Profiler,
  ProfilerBuildError,
};

/// Builds a [`Profiler`] instance from user-provided configuration.
#[derive(Clone, Debug)]
pub struct ProfilerBuilder {
  run_name: String,
  summary_export_enabled: bool,
  trace_export_enabled: bool,
  output_directory: Option<PathBuf>,
  gpu_timestamps_enabled: bool,
  frame_history_limit: usize,
}

impl ProfilerBuilder {
  /// Creates a profiler builder for a named run.
  ///
  /// # Arguments
  /// - `name`: Stable label recorded in exported artifacts.
  ///
  /// # Returns
  /// Returns a builder with placeholder defaults that can be refined by
  /// chaining additional configuration methods.
  pub fn new(name: &str) -> Self {
    return Self {
      run_name: name.to_owned(),
      summary_export_enabled: false,
      trace_export_enabled: false,
      output_directory: None,
      gpu_timestamps_enabled: false,
      frame_history_limit: 120,
    };
  }

  /// Enables or disables summary export for the profiler being built.
  ///
  /// # Arguments
  /// - `enabled`: Whether summary export should be requested.
  ///
  /// # Returns
  /// Returns the updated builder for further configuration.
  pub fn with_summary_export(mut self, enabled: bool) -> Self {
    self.summary_export_enabled = enabled;

    return self;
  }

  /// Enables or disables trace export for the profiler being built.
  ///
  /// # Arguments
  /// - `enabled`: Whether trace export should be requested.
  ///
  /// # Returns
  /// Returns the updated builder for further configuration.
  pub fn with_trace_export(mut self, enabled: bool) -> Self {
    self.trace_export_enabled = enabled;

    return self;
  }

  /// Sets the output directory used for profiler artifacts.
  ///
  /// # Arguments
  /// - `path`: Directory path recorded for future export operations.
  ///
  /// # Returns
  /// Returns the updated builder for further configuration.
  pub fn with_output_directory(mut self, path: &Path) -> Self {
    self.output_directory = Some(path.to_path_buf());

    return self;
  }

  /// Enables or disables GPU timestamp collection for the profiler.
  ///
  /// # Arguments
  /// - `enabled`: Whether GPU timestamps should be requested when supported.
  ///
  /// # Returns
  /// Returns the updated builder for further configuration.
  pub fn with_gpu_timestamps(mut self, enabled: bool) -> Self {
    self.gpu_timestamps_enabled = enabled;

    return self;
  }

  /// Sets the number of frames kept in profiler history.
  ///
  /// # Arguments
  /// - `frames`: Maximum number of frames retained by the profiler.
  ///
  /// # Returns
  /// Returns the updated builder for further configuration.
  pub fn with_frame_history_limit(mut self, frames: usize) -> Self {
    self.frame_history_limit = frames;

    return self;
  }

  /// Builds a placeholder profiler from the current builder state.
  ///
  /// Later implementation steps validate builder inputs and connect the
  /// resulting profiler to collectors and exporters.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns a profiler carrying the current configuration.
  ///
  /// # Errors
  /// Returns [`ProfilerBuildError`] when future validation rejects the builder
  /// configuration.
  pub fn build(self) -> Result<Profiler, ProfilerBuildError> {
    return Ok(Profiler::from_builder(self));
  }

  pub(crate) fn run_name(&self) -> &str {
    return &self.run_name;
  }

  pub(crate) fn summary_export_enabled(&self) -> bool {
    return self.summary_export_enabled;
  }

  pub(crate) fn trace_export_enabled(&self) -> bool {
    return self.trace_export_enabled;
  }

  pub(crate) fn output_directory(&self) -> Option<&Path> {
    return self.output_directory.as_deref();
  }

  pub(crate) fn gpu_timestamps_enabled(&self) -> bool {
    return self.gpu_timestamps_enabled;
  }

  pub(crate) fn frame_history_limit(&self) -> usize {
    return self.frame_history_limit;
  }
}
