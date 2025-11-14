#![allow(clippy::needless_return)]
//! Utility helpers for the `lambda-rs` crate.
//!
//! This module hosts small, reusable helpers that are not part of the public
//! rendering API surface but are useful across modules (e.g., de-duplicated
//! logging of advisories that would otherwise spam every frame).

use std::{
  collections::HashSet,
  sync::{
    Mutex,
    OnceLock,
  },
};

/// Global, process-wide de-duplication set for warn-once messages.
static WARN_ONCE_KEYS: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

/// Log a warning message at most once per unique `key` across the process.
///
/// - `key` SHOULD be stable and descriptive (e.g., include a pipeline label or
///   other identifier) so distinct advisories are tracked independently.
/// - If the internal lock is poisoned, the message is still logged to avoid
///   panics.
pub fn warn_once(key: &str, message: &str) {
  let set = WARN_ONCE_KEYS.get_or_init(|| {
    return Mutex::new(HashSet::new());
  });
  match set.lock() {
    Ok(mut guard) => {
      if guard.insert(key.to_string()) {
        logging::warn!("{}", message);
      }
      return;
    }
    Err(_) => {
      logging::warn!("{}", message);
      return;
    }
  }
}
