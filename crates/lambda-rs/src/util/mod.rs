#![allow(clippy::needless_return)]
//! Utility helpers for the `lambda-rs` crate.
//!
//! This module hosts small, reusable helpers that are not part of the public
//! rendering API surface but are useful across modules (e.g., de-duplicated
//! logging of advisories that would otherwise spam every frame).

use std::{
  collections::{
    HashSet,
    VecDeque,
  },
  sync::{
    Mutex,
    OnceLock,
  },
};

/// Maximum number of unique warn-once keys to retain in memory.
const WARN_ONCE_MAX_KEYS: usize = 1024;

/// Global, process-wide state for warn-once messages.
///
/// The state uses a hash set for membership checks and a queue to track
/// insertion order, allowing the cache to evict the oldest keys when the
/// capacity is reached.
#[derive(Default)]
struct WarnOnceState {
  seen_keys: HashSet<String>,
  key_order: VecDeque<String>,
}

/// Global, process-wide de-duplication cache for warn-once messages.
static WARN_ONCE_KEYS: OnceLock<Mutex<WarnOnceState>> = OnceLock::new();

/// Log a warning message at most once per unique `key` across the process.
///
/// - `key` SHOULD be stable and descriptive (e.g., include a pipeline label or
///   other identifier) so distinct advisories are tracked independently.
/// - If the internal lock is poisoned, the message is still logged to avoid
///   panics.
pub fn warn_once(key: &str, message: &str) {
  let set = WARN_ONCE_KEYS.get_or_init(|| {
    return Mutex::new(WarnOnceState::default());
  });
  match set.lock() {
    Ok(mut guard) => {
      if guard.seen_keys.contains(key) {
        return;
      }

      if guard.seen_keys.len() >= WARN_ONCE_MAX_KEYS {
        if let Some(oldest_key) = guard.key_order.pop_front() {
          guard.seen_keys.remove(&oldest_key);
        }
      }

      let key_string = key.to_string();
      guard.seen_keys.insert(key_string.clone());
      guard.key_order.push_back(key_string);
      logging::warn!("{}", message);
      return;
    }
    Err(_) => {
      logging::warn!("{}", message);
      return;
    }
  }
}
