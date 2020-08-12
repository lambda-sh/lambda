#include "core/OrthographicCameraController.h"

#include "core/events/ApplicationEvent.h"
#include "core/events/Event.h"
#include "core/events/MouseEvent.h"
#include "core/input/Input.h"
#include "core/input/KeyCodes.h"
#include "core/memory/Pointers.h"
#include "core/renderer/OrthographicCamera.h"
#include "core/util/Time.h"

namespace lambda {
namespace core {

OrthographicCameraController::OrthographicCameraController(
    float aspect_ratio, bool can_rotate) :
        aspect_ratio_(aspect_ratio),
        camera_(
            -aspect_ratio_ * zoom_level_,
            aspect_ratio_ * zoom_level_,
            -zoom_level_,
            zoom_level_),
        can_rotate_(can_rotate) {}

void OrthographicCameraController::OnUpdate(util::TimeStep delta) {
    float delta_in_ms = delta.InMilliSeconds<float>();
    if (input::Input::IsKeyPressed(ENGINE_KEY_W)) {
      camera_position_.y += camera_translation_speed_ * delta_in_ms;
    } else if (input::Input::IsKeyPressed(ENGINE_KEY_S)) {
      camera_position_.y -= camera_translation_speed_ * delta_in_ms;
    }

    if (input::Input::IsKeyPressed(ENGINE_KEY_A)) {
      camera_position_.x -= camera_translation_speed_ * delta_in_ms;
    } else if (input::Input::IsKeyPressed(ENGINE_KEY_D)) {
      camera_position_.x += camera_translation_speed_ * delta_in_ms;
    }

    camera_.SetPosition(camera_position_);

    if (can_rotate_) {
      if (input::Input::IsKeyPressed(ENGINE_KEY_Q)) {
        camera_rotation_ -= camera_rotation_speed_  * delta_in_ms;
      } else if (input::Input::IsKeyPressed(ENGINE_KEY_E)) {
        camera_rotation_ += camera_rotation_speed_ * delta_in_ms;
      }

      camera_.SetRotation(camera_rotation_);
    }
}

bool OrthographicCameraController::OnMouseScrolled(
    const events::MouseScrolledEvent& event) {
  zoom_level_ -= event.GetYOffset() * 0.20f;
  camera_.SetProjectionMatrix(
      -aspect_ratio_ * zoom_level_,
      aspect_ratio_ * zoom_level_,
      -zoom_level_,
      zoom_level_);
  return false;
}

bool OrthographicCameraController::OnWindowResize(
    const events::WindowResizeEvent& event) {
  aspect_ratio_ = static_cast<float>(event.GetWidth()) / static_cast<float>(
      event.GetHeight());

  camera_.SetProjectionMatrix(
      -aspect_ratio_ * zoom_level_,
      aspect_ratio_ * zoom_level_,
      -zoom_level_,
      zoom_level_);

  return false;
}

void OrthographicCameraController::OnEvent(
    memory::Shared<events::Event> event) {
  events::EventDispatcher dispatcher(event);

  dispatcher.Dispatch<events::WindowResizeEvent>(
      BIND_EVENT_FN(OrthographicCameraController::OnWindowResize));

  dispatcher.Dispatch<events::MouseScrolledEvent>(
      BIND_EVENT_FN(OrthographicCameraController::OnMouseScrolled));
}

}  // namespace core
}  // namespace lambda
