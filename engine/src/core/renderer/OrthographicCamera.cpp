#include "core/renderer/OrthographicCamera.h"

#include <glad/glad.h>

namespace engine {
namespace renderer {

inline void OrthographicCamera::SetPosition(const glm::vec3& position) {
  position_ = position;
  RecalculateView();
}

inline void OrthographicCamera::SetRotation(float rotation) {
  rotation_ = rotation;
  RecalculateView();
}


}  // namespace renderer
}  // namespace engine
