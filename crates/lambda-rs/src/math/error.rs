//! Math error types returned from fallible operations.

use std::fmt;

/// Errors returned by fallible math operations in `lambda-rs`.
#[derive(Debug, Clone, PartialEq)]
pub enum MathError {
  /// Cross product requires exactly 3 dimensions.
  CrossProductDimension { actual: usize },
  /// Cross product requires both vectors to have the same dimension.
  MismatchedVectorDimensions { left: usize, right: usize },
  /// Rotation axis must be a unit axis vector (one of `[1,0,0]`, `[0,1,0]`,
  /// `[0,0,1]`). A zero axis (`[0,0,0]`) is treated as "no rotation".
  InvalidRotationAxis { axis: [f32; 3] },
  /// Rotation requires a 4x4 matrix.
  InvalidRotationMatrixSize { rows: usize, cols: usize },
  /// Determinant requires a square matrix.
  NonSquareMatrix { rows: usize, cols: usize },
  /// Determinant cannot be computed for an empty matrix.
  EmptyMatrix,
  /// Cannot normalize a zero-length vector.
  ZeroLengthVector,
}

impl fmt::Display for MathError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      MathError::CrossProductDimension { actual } => {
        return write!(f, "Cross product requires 3D vectors, got {}D", actual);
      }
      MathError::MismatchedVectorDimensions { left, right } => {
        return write!(
          f,
          "Vectors must have matching dimensions (left {}D, right {}D)",
          left, right
        );
      }
      MathError::InvalidRotationAxis { axis } => {
        return write!(f, "Rotation axis {:?} is not a unit axis vector", axis);
      }
      MathError::InvalidRotationMatrixSize { rows, cols } => {
        return write!(
          f,
          "Rotation requires a 4x4 matrix, got {}x{}",
          rows, cols
        );
      }
      MathError::NonSquareMatrix { rows, cols } => {
        return write!(
          f,
          "Determinant requires square matrix, got {}x{}",
          rows, cols
        );
      }
      MathError::EmptyMatrix => {
        return write!(f, "Determinant requires a non-empty matrix");
      }
      MathError::ZeroLengthVector => {
        return write!(f, "Cannot normalize a zero-length vector");
      }
    }
  }
}

impl std::error::Error for MathError {}
