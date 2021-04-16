/// @file OrthographicCamera.h
/// @brief A 2D orthographic camera implementation that is compatible with the
/// renderer.
#ifndef LAMBDA_SRC_LAMBDA_CORE_RENDERER_ORTHOGRAPHICCAMERA_H_
#define LAMBDA_SRC_LAMBDA_CORE_RENDERER_ORTHOGRAPHICCAMERA_H_

#include <glm/glm.hpp>
#include <glm/gtc/matrix_transform.hpp>

namespace lambda::core::renderer {

/// @brief An orthographic camera implementation for the 2D engine.
class OrthographicCamera {
 public:
  /// @brief Create a camera with it's left, right, bottom, and top positions
  /// initially defined.
  OrthographicCamera(float left, float right, float bottom, float top);

  /// @brief Get the cameras current rotation.
  const float GetRotation() const { return rotation_; }
  /// @brief Set the cameras rotation.
  void SetRotation(float rotation);

  /// @brief Get the cameras current position. (Stored as a vec3)
  const glm::vec3& GetPosition() const { return position_; }
  /// @brief Set the position of the current camera.
  void SetPosition(const glm::vec3& position);

  /// @brief Get the cameras current projection matrix.
  const glm::mat4& GetProjectionMatrix() const {
      return projection_matrix_; }

  /// @brief Set the projection matrix of the camera by giving the camera new
  /// left, right, bottom, and top positions.
  ///
  /// Recalculates the view matrix.
  void SetProjectionMatrix(float left, float right, float bottom, float top) {
    projection_matrix_ = glm::ortho(left, right, bottom, top, -1.0f, 1.0f);
    RecalculateViewMatrix();
  }

  /// @brief Get the current view matrix.
  const glm::mat4& GetViewMatrix() const { return view_matrix_; }

  /// @brief Get the view projection matrix.
  /// TODO(C3NZ): Implement this to be platform independent. (Need it's own math
  // library?)
  const glm::mat4& GetViewProjectionMatrix() const {
    return view_projection_matrix_;
  }

 private:
  glm::mat4 projection_matrix_;
  glm::mat4 view_matrix_;
  glm::mat4 view_projection_matrix_;

  glm::vec3 position_ = {0.0f, 0.0f, 0.0f};
  float rotation_ = 0.0f;

  void RecalculateViewMatrix();
};

}  // namespace lambda::core::renderer

#endif  // LAMBDA_SRC_LAMBDA_CORE_RENDERER_ORTHOGRAPHICCAMERA_H_
