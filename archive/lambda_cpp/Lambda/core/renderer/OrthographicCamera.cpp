#include "Lambda/core/renderer/OrthographicCamera.h"

#include <glm/glm.hpp>
#include <glm/gtc/matrix_transform.hpp>

namespace lambda::core::renderer {

OrthographicCamera::OrthographicCamera(
    float left, float right, float bottom, float top) :
        projection_matrix_(glm::ortho(left, right, bottom, top, -1.0f, 1.0f)),
        view_matrix_(1.0f) {
  view_projection_matrix_ = projection_matrix_ * view_matrix_;
}

void OrthographicCamera::SetPosition(const glm::vec3& position) {
  position_ = position;
  RecalculateViewMatrix();
}

void OrthographicCamera::SetRotation(const float rotation) {
  rotation_ = rotation;
  RecalculateViewMatrix();
}

void OrthographicCamera::RecalculateViewMatrix() {
  glm::mat4 transform =
      glm::translate(glm::mat4(1.0f), position_) *
      glm::rotate(glm::mat4(1.0f), glm::radians(rotation_), glm::vec3(0, 0, 1));

  view_matrix_ = glm::inverse(transform);
  view_projection_matrix_ = projection_matrix_ * view_matrix_;
}

}  // namespace lambda::core::renderer
