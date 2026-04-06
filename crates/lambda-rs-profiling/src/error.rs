use std::{
  error::Error,
  fmt::{
    Display,
    Formatter,
    Result as FormatResult,
  },
  path::PathBuf,
};

/// Errors produced while building a [`crate::Profiler`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProfilerBuildError {
  /// Indicates that the requested frame history limit is invalid.
  InvalidFrameHistoryLimit { requested: usize },
  /// Indicates that export was requested without an output directory.
  MissingOutputDirectory,
  /// Indicates that the requested output directory is invalid.
  InvalidOutputDirectory { path: PathBuf },
}

impl Display for ProfilerBuildError {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
    match self {
      Self::InvalidFrameHistoryLimit { requested } => {
        return write!(
          formatter,
          "invalid profiler frame history limit: {requested}"
        );
      }
      Self::MissingOutputDirectory => {
        return formatter
          .write_str("profiler export requires an explicit output directory");
      }
      Self::InvalidOutputDirectory { path } => {
        return write!(
          formatter,
          "invalid profiler output directory: {}",
          path.display()
        );
      }
    }
  }
}

impl Error for ProfilerBuildError {}

/// Errors produced while installing a global [`crate::Profiler`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProfilerInstallError {
  /// Indicates that a global profiler is already present.
  AlreadyInstalled,
  /// Indicates that installation is unavailable in the current build stage.
  Unavailable,
}

impl Display for ProfilerInstallError {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
    match self {
      Self::AlreadyInstalled => {
        return formatter.write_str("a profiler is already installed");
      }
      Self::Unavailable => {
        return formatter
          .write_str("global profiler installation is unavailable");
      }
    }
  }
}

impl Error for ProfilerInstallError {}

/// Errors produced while flushing profiler data for a frame.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProfilerFlushError {
  /// Indicates that frame flushing is not yet available.
  Unavailable,
  /// Indicates that export failed during a frame flush.
  Export(ExportError),
}

impl Display for ProfilerFlushError {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
    match self {
      Self::Unavailable => {
        return formatter.write_str("frame flushing is unavailable");
      }
      Self::Export(error) => {
        return write!(
          formatter,
          "frame flushing failed during export: {error}"
        );
      }
    }
  }
}

impl Error for ProfilerFlushError {}

/// Errors produced while exporting profiler artifacts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExportError {
  /// Indicates that the requested export path is invalid or unavailable.
  InvalidOutputDirectory { path: PathBuf },
  /// Indicates that the requested export mode is disabled.
  Disabled { export_name: &'static str },
  /// Indicates that export is not yet implemented.
  Unavailable { export_name: &'static str },
}

impl Display for ExportError {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
    match self {
      Self::InvalidOutputDirectory { path } => {
        return write!(
          formatter,
          "invalid export output directory: {}",
          path.display()
        );
      }
      Self::Disabled { export_name } => {
        return write!(formatter, "{export_name} export is disabled");
      }
      Self::Unavailable { export_name } => {
        return write!(formatter, "{export_name} export is unavailable");
      }
    }
  }
}

impl Error for ExportError {}

/// Errors produced while comparing profiler reports.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompareError {
  /// Indicates that the baseline report did not contain a required scope.
  MissingBaselineScope {
    name: String,
    clock_domain: &'static str,
  },
  /// Indicates that comparison support is not yet implemented.
  Unavailable,
}

impl Display for CompareError {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
    match self {
      Self::MissingBaselineScope { name, clock_domain } => {
        return write!(
          formatter,
          "missing baseline scope `{name}` in `{clock_domain}` clock domain"
        );
      }
      Self::Unavailable => {
        return formatter.write_str("report comparison is unavailable");
      }
    }
  }
}

impl Error for CompareError {}
