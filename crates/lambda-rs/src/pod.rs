//! Plain-old-data marker trait for safe byte uploads.
//!
//! The engine frequently uploads CPU data into GPU buffers by reinterpreting
//! a value or slice as raw bytes. This is only sound for types that are safe
//! to view as bytes.

/// Marker trait for types that are safe to reinterpret as raw bytes.
///
/// This trait is required by typed buffer upload APIs (for example
/// `render::buffer::Buffer::write_value` and `render::buffer::Buffer::write_slice`)
/// and typed buffer creation APIs (for example
/// `render::buffer::BufferBuilder::build`) because those operations upload the
/// in-memory representation of a value to the GPU.
///
/// # Safety
/// Types implementing `PlainOldData` MUST satisfy all of the following:
/// - Every byte of the value is initialized (including any padding bytes).
/// - The type has no pointers or references that would be invalidated by a
///   raw byte copy.
/// - The type's byte representation is stable for GPU consumption. Prefer
///   `#[repr(C)]` or `#[repr(transparent)]`.
///
/// Implementing this trait incorrectly can cause undefined behavior.
pub unsafe trait PlainOldData: Copy {}

unsafe impl PlainOldData for u8 {}
unsafe impl PlainOldData for i8 {}
unsafe impl PlainOldData for u16 {}
unsafe impl PlainOldData for i16 {}
unsafe impl PlainOldData for u32 {}
unsafe impl PlainOldData for i32 {}
unsafe impl PlainOldData for u64 {}
unsafe impl PlainOldData for i64 {}
unsafe impl PlainOldData for u128 {}
unsafe impl PlainOldData for i128 {}
unsafe impl PlainOldData for usize {}
unsafe impl PlainOldData for isize {}
unsafe impl PlainOldData for f32 {}
unsafe impl PlainOldData for f64 {}
unsafe impl PlainOldData for bool {}
unsafe impl PlainOldData for char {}
unsafe impl<T: PlainOldData, const N: usize> PlainOldData for [T; N] {}
