/**
 * @file OrthographicCamera.h
 * @brief A 2D orthographic camera implementation that is compatible with the
 * renderer.
 */
#ifndef ENGINE_SRC_CORE_RENDERER_ORTHOGRAPHICCAMERA_H_
#define ENGINE_SRC_CORE_RENDERER_ORTHOGRAPHICCAMERA_H_

#include <glm/glm.hpp>
#include <glm/gtc/matrix_transform.hpp>

namespace engine {
namespace renderer {

class OrthographicCamera {
 public:
  OrthographicCamera(float left, float right, float bottom, float top);

  const float GetRotation() const { return rotation_; }
  void SetRotation(float rotation);

  const glm::vec3& GetPosition() const { return position_; }
  void SetPosition(const glm::vec3& position);

  const glm::mat4& GetProjectionMatrix() const {
      return projection_matrix_; }
  void SetProjectionMatrix(float left, float right, float bottom, float top) {
    projection_matrix_ = glm::ortho(left, right, bottom, top, -1.0f, 1.0f);
  }
  const glm::mat4& GetViewMatrix() const { return view_matrix_; }
  const glm::mat4& GetViewProjectionMatrix() const {
      return view_projection_matrix_; }

 private:
  glm::mat4 projection_matrix_;
  glm::mat4 view_matrix_;
  glm::mat4 view_projection_matrix_;

  glm::vec3 position_ = {0.0f, 0.0f, 0.0f};
  float rotation_ = 0.0f;

  void RecalculateViewMatrix();
};

}  // namespace renderer
}  // namespace engine

#endif  // ENGINE_SRC_CORE_RENDERER_ORTHOGRAPHICCAMERA_H_

/**
 * @class engine::renderer::OrthographicCamera
 * @brief A 2D orthographic camera implementation that is compatible with the
 * engine.
 */

/**
 * @fn engine::renderer::OrthographicCamera::GetPosition
 * @brief Get the cameras position from origin.
 */
