//! Scene mathematics helpers for common camera and model transforms.
//!
//! These functions centralize the construction of model, view, and projection
//! matrices so that examples and applications do not need to implement the
//! mathematics by hand. All functions use right-handed coordinates and the
//! engine's existing matrix utilities.

use crate::math::{
  matrix,
  matrix::Matrix,
};

/// Convert OpenGL-style normalized device coordinates (Z in [-1, 1]) to
/// wgpu/Vulkan/Direct3D normalized device coordinates (Z in [0, 1]).
///
/// This matrix leaves X and Y unchanged and remaps Z as `z = 0.5 * z + 0.5`.
fn opengl_to_wgpu_ndc() -> [[f32; 4]; 4] {
  return [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 0.5, 0.0],
    [0.0, 0.0, 0.5, 1.0],
  ];
}

/// Simple camera parameters used to produce a perspective projection
/// together with a view translation.
#[derive(Clone, Copy, Debug)]
pub struct SimpleCamera {
  /// World-space camera position used to build a translation view matrix.
  pub position: [f32; 3],
  /// Field of view in turns (where one turn equals 2π radians).
  pub field_of_view_in_turns: f32,
  /// Near clipping plane distance.
  pub near_clipping_plane: f32,
  /// Far clipping plane distance.
  pub far_clipping_plane: f32,
}

/// Compute a model matrix using a translation vector, a rotation axis and
/// an angle in turns, and a uniform scale factor applied around the origin.
pub fn compute_model_matrix(
  translation: [f32; 3],
  rotation_axis: [f32; 3],
  angle_in_turns: f32,
  uniform_scale: f32,
) -> [[f32; 4]; 4] {
  let mut model: [[f32; 4]; 4] = matrix::identity_matrix(4, 4);
  // Apply rotation first, then scaling via a diagonal matrix, and finally translation.
  model = matrix::rotate_matrix(model, rotation_axis, angle_in_turns);

  let mut scaled: [[f32; 4]; 4] = [[0.0; 4]; 4];
  for i in 0..4 {
    for j in 0..4 {
      if i == j {
        scaled[i][j] = if i == 3 { 1.0 } else { uniform_scale };
      } else {
        scaled[i][j] = 0.0;
      }
    }
  }
  model = model.multiply(&scaled);

  let translation_matrix: [[f32; 4]; 4] =
    matrix::translation_matrix(translation);
  return translation_matrix.multiply(&model);
}

/// Compute a model matrix that applies a rotation and uniform scale about a
/// specific local-space pivot point before applying a world-space translation.
///
/// This is useful when your mesh vertices are not centered around the origin
/// and you want to rotate the object "in place" (around its own center) rather
/// than orbit around the world origin.
pub fn compute_model_matrix_about_pivot(
  translation: [f32; 3],
  rotation_axis: [f32; 3],
  angle_in_turns: f32,
  uniform_scale: f32,
  pivot_local: [f32; 3],
) -> [[f32; 4]; 4] {
  // Base rotation and scale around origin.
  let base = compute_model_matrix(
    [0.0, 0.0, 0.0],
    rotation_axis,
    angle_in_turns,
    uniform_scale,
  );
  // Translate to pivot, apply base, then translate back from pivot.
  let to_pivot: [[f32; 4]; 4] = matrix::translation_matrix(pivot_local);
  let from_pivot: [[f32; 4]; 4] = matrix::translation_matrix([
    -pivot_local[0],
    -pivot_local[1],
    -pivot_local[2],
  ]);
  // World translation after rotating around the pivot.
  let world_translation: [[f32; 4]; 4] =
    matrix::translation_matrix(translation);

  // For column-vector convention: T_world * ( T_pivot * (R*S) * T_-pivot )
  return world_translation
    .multiply(&to_pivot)
    .multiply(&base)
    .multiply(&from_pivot);
}

/// Compute a simple view matrix from a camera position.
///
/// The view matrix is the inverse of the camera transform. For a camera whose
/// world-space transform is a pure translation by `camera_position`, the
/// inverse is a translation by the negated vector. This function applies that
/// inverse so the world moves opposite to the camera when rendering.
pub fn compute_view_matrix(camera_position: [f32; 3]) -> [[f32; 4]; 4] {
  let inverse = [
    -camera_position[0],
    -camera_position[1],
    -camera_position[2],
  ];
  return matrix::translation_matrix(inverse);
}

/// Compute a perspective projection matrix from camera parameters and the
/// current viewport width and height.
pub fn compute_perspective_projection(
  field_of_view_in_turns: f32,
  viewport_width: u32,
  viewport_height: u32,
  near_clipping_plane: f32,
  far_clipping_plane: f32,
) -> [[f32; 4]; 4] {
  let aspect_ratio = viewport_width as f32 / viewport_height as f32;
  // Build an OpenGL-style projection (Z in [-1, 1]) using the existing math
  // helper, then convert to wgpu/Vulkan/D3D NDC (Z in [0, 1]).
  let projection_gl: [[f32; 4]; 4] = matrix::perspective_matrix(
    field_of_view_in_turns,
    aspect_ratio,
    near_clipping_plane,
    far_clipping_plane,
  );
  let conversion = opengl_to_wgpu_ndc();
  return conversion.multiply(&projection_gl);
}

/// Compute a full model‑view‑projection matrix given a simple camera, a
/// viewport, and the model transform parameters.
///
/// Example
/// ```rust
/// use lambda::render::scene_math::{SimpleCamera, compute_model_view_projection_matrix};
/// let camera = SimpleCamera {
///   position: [0.0, 0.0, 3.0],
///   field_of_view_in_turns: 0.25,
///   near_clipping_plane: 0.01,
///   far_clipping_plane: 100.0,
/// };
/// let mvp = compute_model_view_projection_matrix(
///   &camera,
///   800,
///   600,
///   [0.0, 0.0, 0.0],
///   [0.0, 1.0, 0.0],
///   0.0,
///   1.0,
/// );
/// ```
pub fn compute_model_view_projection_matrix(
  camera: &SimpleCamera,
  viewport_width: u32,
  viewport_height: u32,
  model_translation: [f32; 3],
  rotation_axis: [f32; 3],
  angle_in_turns: f32,
  uniform_scale: f32,
) -> [[f32; 4]; 4] {
  let model = compute_model_matrix(
    model_translation,
    rotation_axis,
    angle_in_turns,
    uniform_scale,
  );
  let view = compute_view_matrix(camera.position);
  let projection = compute_perspective_projection(
    camera.field_of_view_in_turns,
    viewport_width,
    viewport_height,
    camera.near_clipping_plane,
    camera.far_clipping_plane,
  );
  return projection.multiply(&view).multiply(&model);
}

/// Compute a full model-view-projection matrix for a rotation around a specific
/// local-space pivot point.
pub fn compute_model_view_projection_matrix_about_pivot(
  camera: &SimpleCamera,
  viewport_width: u32,
  viewport_height: u32,
  model_translation: [f32; 3],
  rotation_axis: [f32; 3],
  angle_in_turns: f32,
  uniform_scale: f32,
  pivot_local: [f32; 3],
) -> [[f32; 4]; 4] {
  let model = compute_model_matrix_about_pivot(
    model_translation,
    rotation_axis,
    angle_in_turns,
    uniform_scale,
    pivot_local,
  );
  let view = compute_view_matrix(camera.position);
  let projection = compute_perspective_projection(
    camera.field_of_view_in_turns,
    viewport_width,
    viewport_height,
    camera.near_clipping_plane,
    camera.far_clipping_plane,
  );
  return projection.multiply(&view).multiply(&model);
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::math::matrix as m;

  /// This test demonstrates the complete order of operations used to produce a
  /// model matrix. It rotates a base identity matrix by a chosen axis and
  /// angle, applies a uniform scale using a diagonal scaling matrix, and then
  /// applies a world translation. The expected matrix is built step by step in
  /// the same manner so that individual differences are easy to reason about
  /// when reading a failure. Every element is compared using a small tolerance
  /// to account for floating point rounding.
  #[test]
  fn model_matrix_composes_rotation_scale_and_translation() {
    let translation = [3.0, -2.0, 5.0];
    let axis = [0.0, 1.0, 0.0];
    let angle_in_turns = 0.25; // quarter turn
    let scale = 2.0;

    // Compute via the public function under test.
    let actual = compute_model_matrix(translation, axis, angle_in_turns, scale);

    // Build the expected matrix explicitly: R, then S, then T.
    let mut expected: [[f32; 4]; 4] = m::identity_matrix(4, 4);
    expected = m::rotate_matrix(expected, axis, angle_in_turns);

    let mut s: [[f32; 4]; 4] = [[0.0; 4]; 4];
    for i in 0..4 {
      for j in 0..4 {
        s[i][j] = if i == j {
          if i == 3 {
            1.0
          } else {
            scale
          }
        } else {
          0.0
        };
      }
    }
    expected = expected.multiply(&s);

    let t: [[f32; 4]; 4] = m::translation_matrix(translation);
    let expected = t.multiply(&expected);

    for i in 0..4 {
      for j in 0..4 {
        crate::assert_approximately_equal!(
          actual.at(i, j),
          expected.at(i, j),
          1e-5
        );
      }
    }
  }

  /// This test verifies that rotating and scaling around a local pivot point
  /// produces the same result as translating into the pivot, applying the base
  /// transform, and translating back out, followed by the world translation.
  /// It constructs both forms and checks that all elements match within a
  /// small tolerance.
  #[test]
  fn model_matrix_respects_local_pivot() {
    let translation = [1.0, 2.0, 3.0];
    let axis = [1.0, 0.0, 0.0];
    let angle_in_turns = 0.125; // one eighth of a full turn
    let scale = 0.5;
    let pivot = [10.0, -4.0, 2.0];

    let actual = compute_model_matrix_about_pivot(
      translation,
      axis,
      angle_in_turns,
      scale,
      pivot,
    );

    let base =
      compute_model_matrix([0.0, 0.0, 0.0], axis, angle_in_turns, scale);
    let to_pivot: [[f32; 4]; 4] = m::translation_matrix(pivot);
    let from_pivot: [[f32; 4]; 4] =
      m::translation_matrix([-pivot[0], -pivot[1], -pivot[2]]);
    let world: [[f32; 4]; 4] = m::translation_matrix(translation);
    let expected = world
      .multiply(&to_pivot)
      .multiply(&base)
      .multiply(&from_pivot);

    for i in 0..4 {
      for j in 0..4 {
        crate::assert_approximately_equal!(
          actual.at(i, j),
          expected.at(i, j),
          1e-5
        );
      }
    }
  }

  /// This test confirms that the view computation is the inverse of a camera
  /// translation. For a camera expressed only as a world space translation, the
  /// inverse is a translation by the negated vector. The test constructs that
  /// expected matrix directly and compares it to the function result.
  #[test]
  fn view_matrix_is_inverse_translation() {
    let camera_position = [7.0, -3.0, 2.5];
    let expected: [[f32; 4]; 4] = m::translation_matrix([
      -camera_position[0],
      -camera_position[1],
      -camera_position[2],
    ]);
    let actual = compute_view_matrix(camera_position);
    assert_eq!(actual, expected);
  }

  /// This test validates that the perspective projection matches an
  /// OpenGL‑style projection that is converted into the normalized device
  /// coordinate range used by the target platforms. The expected conversion is
  /// performed by multiplying a fixed conversion matrix with the projection
  /// produced by the existing matrix helper. The result is compared element by
  /// element within a small tolerance.
  #[test]
  fn perspective_projection_matches_converted_reference() {
    let fov_turns = 0.25;
    let width = 1280;
    let height = 720;
    let near = 0.1;
    let far = 100.0;

    let actual =
      compute_perspective_projection(fov_turns, width, height, near, far);

    let aspect = width as f32 / height as f32;
    let projection_gl: [[f32; 4]; 4] =
      m::perspective_matrix(fov_turns, aspect, near, far);
    let conversion = [
      [1.0, 0.0, 0.0, 0.0],
      [0.0, 1.0, 0.0, 0.0],
      [0.0, 0.0, 0.5, 0.0],
      [0.0, 0.0, 0.5, 1.0],
    ];
    let expected = conversion.multiply(&projection_gl);

    for i in 0..4 {
      for j in 0..4 {
        crate::assert_approximately_equal!(
          actual.at(i, j),
          expected.at(i, j),
          1e-5
        );
      }
    }
  }

  /// This test builds a full model, view, and projection composition using both
  /// the public helper and a reference expression that multiplies the same
  /// parts in the same order. It uses a simple camera and a non‑trivial model
  /// transform to provide coverage for the code paths. Results are compared
  /// with a small tolerance to account for floating point rounding.
  #[test]
  fn model_view_projection_composition_matches_reference() {
    let camera = SimpleCamera {
      position: [0.5, -1.0, 2.0],
      field_of_view_in_turns: 0.3,
      near_clipping_plane: 0.01,
      far_clipping_plane: 500.0,
    };
    let (w, h) = (1024, 600);
    let model_t = [2.0, 0.5, -3.0];
    let axis = [0.0, 0.0, 1.0];
    let angle = 0.2;
    let scale = 1.25;

    let actual = compute_model_view_projection_matrix(
      &camera, w, h, model_t, axis, angle, scale,
    );

    let model = compute_model_matrix(model_t, axis, angle, scale);
    let view = compute_view_matrix(camera.position);
    let proj = compute_perspective_projection(
      camera.field_of_view_in_turns,
      w,
      h,
      camera.near_clipping_plane,
      camera.far_clipping_plane,
    );
    let expected = proj.multiply(&view).multiply(&model);

    for i in 0..4 {
      for j in 0..4 {
        crate::assert_approximately_equal!(
          actual.at(i, j),
          expected.at(i, j),
          1e-5
        );
      }
    }
  }

  /// This test builds a full model, view, and projection composition for a
  /// model that rotates and scales around a local pivot point. It compares the
  /// public helper result to a reference expression that expands the pivot
  /// operations into individual translations and the base transform. Elements
  /// are compared with a small tolerance to make the test robust to floating
  /// point differences.
  #[test]
  fn model_view_projection_about_pivot_matches_reference() {
    let camera = SimpleCamera {
      position: [-3.0, 0.0, 1.0],
      field_of_view_in_turns: 0.15,
      near_clipping_plane: 0.1,
      far_clipping_plane: 50.0,
    };
    let (w, h) = (800, 480);
    let model_t = [0.0, -1.0, 2.0];
    let axis = [0.0, 1.0, 0.0];
    let angle = 0.4;
    let scale = 0.75;
    let pivot = [5.0, 0.0, -2.0];

    let actual = compute_model_view_projection_matrix_about_pivot(
      &camera, w, h, model_t, axis, angle, scale, pivot,
    );

    let base = compute_model_matrix([0.0, 0.0, 0.0], axis, angle, scale);
    let to_pivot: [[f32; 4]; 4] = m::translation_matrix(pivot);
    let from_pivot: [[f32; 4]; 4] =
      m::translation_matrix([-pivot[0], -pivot[1], -pivot[2]]);
    let world: [[f32; 4]; 4] = m::translation_matrix(model_t);
    let model = world
      .multiply(&to_pivot)
      .multiply(&base)
      .multiply(&from_pivot);
    let view = compute_view_matrix(camera.position);
    let proj = compute_perspective_projection(
      camera.field_of_view_in_turns,
      w,
      h,
      camera.near_clipping_plane,
      camera.far_clipping_plane,
    );
    let expected = proj.multiply(&view).multiply(&model);

    for i in 0..4 {
      for j in 0..4 {
        crate::assert_approximately_equal!(
          actual.at(i, j),
          expected.at(i, j),
          1e-5
        );
      }
    }
  }
}
