//! CPU-only profiling regression fixtures.

#![cfg(feature = "profiling")]

use std::{
  env,
  fs,
  path::{
    Path,
    PathBuf,
  },
  process,
  sync::atomic::{
    AtomicU64,
    Ordering,
  },
};

use lambda::profiling::{
  ComparisonTolerances,
  ProfilerBuilder,
  SummaryReport,
};

static NEXT_TEST_ID: AtomicU64 = AtomicU64::new(0);

/// Returns a unique temporary directory path for one profiling regression test.
fn unique_test_path(suffix: &str) -> PathBuf {
  let test_id = NEXT_TEST_ID.fetch_add(1, Ordering::Relaxed);

  return env::temp_dir().join(format!(
    "lambda-rs-profiling-regression-{suffix}-{}-{test_id}",
    process::id()
  ));
}

/// Removes one filesystem path when it already exists.
fn remove_path_if_present(path: &Path) {
  if !path.exists() {
    return;
  }

  if path.is_dir() {
    fs::remove_dir_all(path).unwrap();

    return;
  }

  fs::remove_file(path).unwrap();
}

/// Returns the checked-in baseline path for the CPU regression fixture.
fn cpu_baseline_path() -> PathBuf {
  return Path::new(env!("CARGO_MANIFEST_DIR"))
    .join("tests/fixtures/profiling-regression-cpu-baseline.json");
}

#[test]
fn profiling_regression_cpu_fixture_matches_checked_in_baseline() {
  let output_directory = unique_test_path("cpu");
  remove_path_if_present(&output_directory);
  let profiler = ProfilerBuilder::new("tests-profiling-regression-cpu")
    .with_summary_export(true)
    .with_output_directory(&output_directory)
    .with_frame_history_limit(8)
    .build()
    .unwrap();

  for frame_index in 0..4_u64 {
    {
      let _scope = profiler.begin_scope("tests.profiling_regression.work");
      let mut accumulator = 0_u64;

      for iteration in 0..25_000_u64 {
        accumulator = accumulator.wrapping_add(std::hint::black_box(
          iteration.wrapping_add(frame_index),
        ));
      }

      std::hint::black_box(accumulator);
    }

    profiler.record_counter("tests.profiling_regression.frames", frame_index);
    let artifacts = profiler.flush_frame().unwrap();

    assert!(artifacts.summary_path.is_some());
    assert_eq!(artifacts.trace_path, None);
  }

  let current_summary_bytes = fs::read(
    output_directory.join("tests-profiling-regression-cpu-summary.json"),
  )
  .unwrap();
  let baseline_summary_bytes = fs::read(cpu_baseline_path()).unwrap();
  let current_summary =
    SummaryReport::from_json_bytes(&current_summary_bytes).unwrap();
  let baseline_summary =
    SummaryReport::from_json_bytes(&baseline_summary_bytes).unwrap();
  let comparison = current_summary
    .compare_against(&baseline_summary, &ComparisonTolerances::cpu_defaults());

  assert!(comparison.passed());
  assert_eq!(comparison.matched_scope_count(), 2);
  assert_eq!(comparison.violation_count(), 0);
  assert_eq!(comparison.missing_current_scopes().len(), 0);
  assert_eq!(comparison.extra_current_scopes().len(), 0);
  assert_eq!(
    current_summary
      .require_scope("tests.profiling_regression.work", "cpu")
      .unwrap()
      .clock_domain(),
    "cpu"
  );

  remove_path_if_present(&output_directory);
}
