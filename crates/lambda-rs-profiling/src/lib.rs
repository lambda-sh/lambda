#![allow(clippy::needless_return)]
//! Profiling support types and macros for Lambda.
//!
//! This crate is re-exported by `lambda-rs` as `lambda::profiling` when one or
//! more profiling features are enabled. The current implementation provides the
//! public API, CPU event collection, and user-facing macros required for later
//! export and regression-comparison work.

mod builder;
mod collector;
mod error;
mod macros;
mod profiler;
mod report;

pub use builder::ProfilerBuilder;
pub use error::{
  CompareError,
  ExportError,
  ProfilerBuildError,
  ProfilerFlushError,
  ProfilerInstallError,
};
pub use profiler::{
  FrameArtifacts,
  Profiler,
  ScopeGuard,
};
pub use report::{
  ComparisonReport,
  SummaryReport,
  TraceReport,
};

/// Internal helpers used by public macros.
#[doc(hidden)]
pub mod __private {
  use crate::{
    Profiler,
    ScopeGuard,
  };

  /// Returns the current profiler guard or an inert placeholder guard.
  pub fn begin_scope(name: &'static str) -> ScopeGuard {
    if let Some(profiler) = Profiler::global() {
      return profiler.begin_scope(name);
    }

    return ScopeGuard::disabled(name);
  }

  /// Records a counter when a profiler is globally installed.
  pub fn record_counter(name: &'static str, value: u64) {
    if let Some(profiler) = Profiler::global() {
      profiler.record_counter(name, value);
    }
  }

  /// Records an instant when a profiler is globally installed.
  pub fn record_instant(name: &'static str) {
    if let Some(profiler) = Profiler::global() {
      profiler.record_instant(name);
    }
  }

  /// Returns a stable fallback scope name for `profile_function!`.
  pub const fn function_scope_name(module_path: &'static str) -> &'static str {
    return module_path;
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    profile_counter,
    profile_function,
    profile_instant,
    profile_scope,
  };

  #[test]
  fn profiling_macros_compile_without_installed_profiler() {
    let _guard = profile_scope!("tests.compile_scope");
    profile_counter!("tests.counter", 7_u64);
    profile_instant!("tests.instant");
  }

  #[test]
  fn profile_scope_returns_bindable_guard_value() {
    let guard = profile_scope!("tests.bindable_scope");

    assert_eq!(guard.name(), "tests.bindable_scope");
    assert!(!guard.is_closed());
  }

  #[test]
  fn profile_function_uses_a_stable_module_qualified_fallback() {
    let guard = profile_function!();

    assert_eq!(guard.name(), module_path!());
  }
}
