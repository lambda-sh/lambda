#ifndef LAMBDA_SRC_LAMBDA_CORE_ORTHOGRAPHICCAMERACONTROLLER_H_
#define LAMBDA_SRC_LAMBDA_CORE_ORTHOGRAPHICCAMERACONTROLLER_H_

#include "Lambda/core/events/ApplicationEvent.h"
#include "Lambda/core/events/MouseEvent.h"
#include "Lambda/core/memory/Pointers.h"
#include "Lambda/core/renderer/OrthographicCamera.h"
#include "Lambda/lib/Time.h"

namespace lambda::core {

class OrthographicCameraController {
 public:
  explicit OrthographicCameraController(
      float aspect_ratio, bool can_rotate = false);

  void OnUpdate(lib::TimeStep delta);
  void OnEvent(events::Event* const event);

  const renderer::OrthographicCamera& GetOrthographicCamera() const {
      return camera_; }
 private:
  float aspect_ratio_;
  bool can_rotate_;
  float zoom_level_ = 1.0f;

  renderer::OrthographicCamera camera_;
  glm::vec3 camera_position_ = {0.0f, 0.0f, 0.0f};
  float camera_translation_speed_ = 0.01f;
  float camera_rotation_speed_ =  0.03f;
  float camera_rotation_ = 0.0f;

  bool OnMouseScrolled(const events::MouseScrolledEvent& event);
  bool OnWindowResize(const events::WindowResizeEvent& event);
};

}  // namespace lambda::core

#endif  // LAMBDA_SRC_LAMBDA_CORE_ORTHOGRAPHICCAMERACONTROLLER_H_
