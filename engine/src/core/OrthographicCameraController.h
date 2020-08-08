#ifndef ENGINE_SRC_CORE_ORTHOGRAPHICCAMERACONTROLLER_H_
#define ENGINE_SRC_CORE_ORTHOGRAPHICCAMERACONTROLLER_H_

#include "core/events/ApplicationEvent.h"
#include "core/events/MouseEvent.h"
#include "core/memory/Pointers.h"
#include "core/renderer/OrthographicCamera.h"
#include "core/util/Time.h"

namespace engine {

using engine::events::Event;
using engine::events::MouseScrolledEvent;
using engine::events::WindowResizeEvent;
using engine::memory::Shared;
using engine::renderer::OrthographicCamera;
using engine::util::TimeStep;

class OrthographicCameraController {
 public:
  explicit OrthographicCameraController(
      float aspect_ratio, bool can_rotate = false);

  void OnUpdate(TimeStep delta);
  void OnEvent(Shared<Event> event);

  const OrthographicCamera& GetOrthographicCamera() { return camera_; }
 private:
  float aspect_ratio_;
  bool can_rotate_;
  float zoom_level_ = 1.0f;

  OrthographicCamera camera_;
  glm::vec3 camera_position_ = {0.0f, 0.0f, 0.0f};
  float camera_translation_speed_ = 0.01f;
  float camera_rotation_speed_ =  0.03f;
  float camera_rotation_ = 0.0f;

  bool OnMouseScrolled(const MouseScrolledEvent& event);
  bool OnWindowResize(const WindowResizeEvent& event);
};

}  // namespace engine

#endif  // ENGINE_SRC_CORE_ORTHOGRAPHICCAMERACONTROLLER_H_
