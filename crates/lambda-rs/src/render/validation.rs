//! Small helpers for limits and alignment validation used by the renderer.

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
}
