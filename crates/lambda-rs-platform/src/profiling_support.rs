/// Re-exports third-party support crates used by profiling integrations.
pub mod deps {
  /// Re-exported `serde` crate for profiling support crates.
  pub use serde;
  /// Re-exported `serde_json` crate for profiling support crates.
  pub use serde_json;
  /// Re-exported `time` crate for profiling support crates.
  pub use time;
}
