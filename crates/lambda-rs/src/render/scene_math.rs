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
  [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 0.5, 0.0],
    [0.0, 0.0, 0.5, 1.0],
  ]
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
  translation_matrix.multiply(&model)
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
  world_translation
    .multiply(&to_pivot)
    .multiply(&base)
    .multiply(&from_pivot)
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
  matrix::translation_matrix(inverse)
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
  conversion.multiply(&projection_gl)
}

/// Compute a full model-view-projection matrix given a simple camera, a
/// viewport, and the model transform parameters.
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
  projection.multiply(&view).multiply(&model)
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
  projection.multiply(&view).multiply(&model)
}
