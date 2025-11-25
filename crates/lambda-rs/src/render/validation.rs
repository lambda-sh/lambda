//! Small helpers for limits and alignment validation used by the renderer.

use std::{
  collections::HashSet,
  ops::Range,
};

/// Align `value` up to the nearest multiple of `align`.
/// If `align` is zero, returns `value` unchanged.
pub fn align_up(value: u64, align: u64) -> u64 {
  if align == 0 {
    return value;
  }
  let mask = align - 1;
  return (value + mask) & !mask;
}

/// Validate a set of dynamic offsets against the required count and alignment.
/// Returns `Ok(())` when valid; otherwise a human‑readable error message.
pub fn validate_dynamic_offsets(
  required_count: u32,
  offsets: &[u32],
  alignment: u32,
  set_index: u32,
) -> Result<(), String> {
  if offsets.len() as u32 != required_count {
    return Err(format!(
      "Bind group at set {} expects {} dynamic offsets, got {}",
      set_index,
      required_count,
      offsets.len()
    ));
  }
  let align = alignment.max(1);
  for (i, off) in offsets.iter().enumerate() {
    if *off % align != 0 {
      return Err(format!(
        "Dynamic offset[{}]={} is not {}-byte aligned",
        i, off, align
      ));
    }
  }
  return Ok(());
}

/// Validate that a multi-sample count is supported by the engine.
///
/// Allowed counts are 1, 2, 4, and 8. Higher or non-power-of-two values are
/// rejected at this layer to provide consistent behavior across platforms.
pub fn validate_sample_count(samples: u32) -> Result<(), String> {
  match samples {
    1 | 2 | 4 | 8 => return Ok(()),
    other => {
      return Err(format!(
        "Unsupported multi-sample count {} (allowed: 1, 2, 4, 8)",
        other
      ));
    }
  }
}

/// Validate that an instance range is well-formed for a draw command.
///
/// The `command_name` is included in any error message to make diagnostics
/// easier to interpret when multiple draw commands are present.
pub fn validate_instance_range(
  command_name: &str,
  instances: &Range<u32>,
) -> Result<(), String> {
  if instances.start > instances.end {
    return Err(format!(
      "{} instance range start {} is greater than end {}",
      command_name, instances.start, instances.end
    ));
  }
  return Ok(());
}

/// Validate that all per-instance vertex buffer slots have been bound before
/// issuing a draw that consumes them.
///
/// The `pipeline_label` identifies the pipeline in diagnostics. The
/// `per_instance_slots` slice marks which vertex buffer slots advance once
/// per instance, while `bound_slots` tracks the vertex buffer slots that
/// have been bound in the current render pass.
pub fn validate_instance_bindings(
  pipeline_label: &str,
  per_instance_slots: &[bool],
  bound_slots: &HashSet<u32>,
) -> Result<(), String> {
  for (slot, is_instance) in per_instance_slots.iter().enumerate() {
    if *is_instance && !bound_slots.contains(&(slot as u32)) {
      return Err(format!(
        "Render pipeline '{}' requires a per-instance vertex buffer bound at slot {} but no BindVertexBuffer command bound that slot in this pass",
        pipeline_label,
        slot
      ));
    }
  }
  return Ok(());
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn align_up_noop_on_zero_align() {
    assert_eq!(align_up(13, 0), 13);
  }

  #[test]
  fn align_up_rounds_to_multiple() {
    assert_eq!(align_up(0, 256), 0);
    assert_eq!(align_up(1, 256), 256);
    assert_eq!(align_up(255, 256), 256);
    assert_eq!(align_up(256, 256), 256);
    assert_eq!(align_up(257, 256), 512);
  }

  #[test]
  fn validate_dynamic_offsets_count_and_alignment() {
    // Correct count and alignment
    assert!(validate_dynamic_offsets(2, &[0, 256], 256, 0).is_ok());

    // Wrong count
    let err = validate_dynamic_offsets(3, &[0, 256], 256, 1)
      .err()
      .unwrap();
    assert!(err.contains("expects 3 dynamic offsets"));

    // Misaligned
    let err = validate_dynamic_offsets(2, &[0, 128], 256, 0)
      .err()
      .unwrap();
    assert!(err.contains("not 256-byte aligned"));
  }

  #[test]
  fn validate_instance_range_accepts_valid_ranges() {
    assert!(validate_instance_range("Draw", &(0..1)).is_ok());
    assert!(validate_instance_range("DrawIndexed", &(2..2)).is_ok());
  }

  #[test]
  fn validate_instance_range_rejects_negative_length() {
    let err = validate_instance_range("Draw", &(5..1))
      .err()
      .expect("must error");
    assert!(err.contains("Draw instance range start 5 is greater than end 1"));
  }

  #[test]
  fn validate_instance_bindings_accepts_bound_slots() {
    let per_instance_slots = vec![true, false, true];
    let mut bound = HashSet::new();
    bound.insert(0);
    bound.insert(2);

    assert!(validate_instance_bindings(
      "test-pipeline",
      &per_instance_slots,
      &bound
    )
    .is_ok());
  }

  #[test]
  fn validate_instance_bindings_rejects_missing_slot() {
    let per_instance_slots = vec![true, false, true];
    let mut bound = HashSet::new();
    bound.insert(0);

    let err =
      validate_instance_bindings("instanced", &per_instance_slots, &bound)
        .err()
        .expect("must error");
    assert!(err.contains("instanced"));
    assert!(err.contains("slot 2"));
  }
}
