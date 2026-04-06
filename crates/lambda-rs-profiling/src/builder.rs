use std::{
  fs,
  path::{
    Path,
    PathBuf,
  },
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
  /// - `path`: Directory path recorded for future export operations. The
  ///   directory is created during [`Self::build`] when it does not already
  ///   exist.
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
  /// - `frames`: Maximum number of frames retained by the profiler. Values
  ///   less than `1` are rejected during [`Self::build`].
  ///
  /// # Returns
  /// Returns the updated builder for further configuration.
  pub fn with_frame_history_limit(mut self, frames: usize) -> Self {
    self.frame_history_limit = frames;

    return self;
  }

  /// Builds a profiler from the current builder state.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns a profiler carrying the validated current configuration.
  ///
  /// # Errors
  /// Returns [`ProfilerBuildError`] when the frame history limit is invalid,
  /// export is enabled without an output directory, or the output directory
  /// cannot be created or resolved to a directory.
  pub fn build(self) -> Result<Profiler, ProfilerBuildError> {
    self.validate()?;

    return Ok(Profiler::from_builder(self));
  }

  /// Validates the builder state before construction.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns `Ok(())` when the builder state is internally consistent.
  ///
  /// # Errors
  /// Returns [`ProfilerBuildError`] when one or more builder settings are not
  /// valid for profiler construction.
  fn validate(&self) -> Result<(), ProfilerBuildError> {
    self.validate_frame_history_limit()?;
    self.validate_output_directory()?;

    return Ok(());
  }

  /// Validates the configured frame history limit.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns `Ok(())` when the configured frame history limit is valid.
  ///
  /// # Errors
  /// Returns [`ProfilerBuildError::InvalidFrameHistoryLimit`] when the
  /// configured frame history limit is zero.
  fn validate_frame_history_limit(&self) -> Result<(), ProfilerBuildError> {
    if self.frame_history_limit == 0 {
      return Err(ProfilerBuildError::InvalidFrameHistoryLimit {
        requested: self.frame_history_limit,
      });
    }

    return Ok(());
  }

  /// Validates and prepares the configured output directory.
  ///
  /// # Arguments
  /// This method does not take additional arguments.
  ///
  /// # Returns
  /// Returns `Ok(())` when the output directory configuration is valid.
  ///
  /// # Errors
  /// Returns [`ProfilerBuildError::MissingOutputDirectory`] when export is
  /// enabled without a destination and
  /// [`ProfilerBuildError::InvalidOutputDirectory`] when the configured path
  /// cannot be created or does not resolve to a directory.
  fn validate_output_directory(&self) -> Result<(), ProfilerBuildError> {
    if (self.summary_export_enabled || self.trace_export_enabled)
      && self.output_directory.is_none()
    {
      return Err(ProfilerBuildError::MissingOutputDirectory);
    }

    if let Some(path) = self.output_directory() {
      fs::create_dir_all(path).map_err(|_| {
        return ProfilerBuildError::InvalidOutputDirectory {
          path: path.to_path_buf(),
        };
      })?;

      if !path.is_dir() {
        return Err(ProfilerBuildError::InvalidOutputDirectory {
          path: path.to_path_buf(),
        });
      }
    }

    return Ok(());
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

#[cfg(test)]
mod builder_tests {
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

  use super::ProfilerBuilder;
  use crate::ProfilerBuildError;

  static NEXT_TEST_ID: AtomicU64 = AtomicU64::new(0);

  fn unique_test_path(suffix: &str) -> PathBuf {
    let test_id = NEXT_TEST_ID.fetch_add(1, Ordering::Relaxed);

    return env::temp_dir().join(format!(
      "lambda-rs-profiling-{suffix}-{}-{test_id}",
      process::id()
    ));
  }

  fn remove_path_if_present(path: &PathBuf) {
    if !path.exists() {
      return;
    }

    if path.is_dir() {
      fs::remove_dir_all(path).unwrap();

      return;
    }

    fs::remove_file(path).unwrap();
  }

  #[test]
  fn builder_rejects_zero_frame_history_limit() {
    let error = ProfilerBuilder::new("tests.builder.invalid_frame_history")
      .with_frame_history_limit(0)
      .build()
      .unwrap_err();

    assert_eq!(
      error,
      ProfilerBuildError::InvalidFrameHistoryLimit { requested: 0 }
    );
  }

  #[test]
  fn builder_rejects_export_without_output_directory() {
    let error = ProfilerBuilder::new("tests.builder.missing_output_directory")
      .with_summary_export(true)
      .build()
      .unwrap_err();

    assert_eq!(error, ProfilerBuildError::MissingOutputDirectory);
  }

  #[test]
  fn builder_rejects_output_directory_that_is_a_file() {
    let path = unique_test_path("builder-invalid-output");
    remove_path_if_present(&path);
    fs::write(&path, "not a directory").unwrap();

    let error = ProfilerBuilder::new("tests.builder.invalid_output_directory")
      .with_output_directory(&path)
      .build()
      .unwrap_err();

    assert_eq!(
      error,
      ProfilerBuildError::InvalidOutputDirectory { path: path.clone() }
    );

    remove_path_if_present(&path);
  }

  #[test]
  fn builder_creates_missing_output_directory() {
    let path = unique_test_path("builder-create-output");
    remove_path_if_present(&path);

    let profiler =
      ProfilerBuilder::new("tests.builder.create_output_directory")
        .with_output_directory(&path)
        .build();

    assert!(profiler.is_ok());
    assert!(path.is_dir());

    remove_path_if_present(&path);
  }
}
