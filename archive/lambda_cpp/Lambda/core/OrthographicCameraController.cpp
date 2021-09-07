#include "Lambda/core/OrthographicCameraController.h"

#include "Lambda/core/events/ApplicationEvent.h"
#include "Lambda/core/events/Event.h"
#include "Lambda/core/events/MouseEvent.h"
#include "Lambda/core/input/Input.h"
#include "Lambda/core/input/KeyCodes.h"
#include "Lambda/core/memory/Pointers.h"
#include "Lambda/core/renderer/OrthographicCamera.h"
#include "Lambda/lib/Time.h"

namespace lambda::core {

OrthographicCameraController::OrthographicCameraController(
    const float aspect_ratio, const bool can_rotate) :
        aspect_ratio_(aspect_ratio),
        can_rotate_(can_rotate),
        camera_(
            -aspect_ratio_ * zoom_level_,
            aspect_ratio_ * zoom_level_,
            -zoom_level_,
            zoom_level_) {}

void OrthographicCameraController::OnUpdate(lib::TimeStep delta) {
    const float delta_in_ms = delta.InMilliseconds<float>();
    if (input::Input::IsKeyPressed(LAMBDA_KEY_W)) {
      camera_position_.y += camera_translation_speed_ * delta_in_ms;
    } else if (input::Input::IsKeyPressed(LAMBDA_KEY_S)) {
      camera_position_.y -= camera_translation_speed_ * delta_in_ms;
    }

    if (input::Input::IsKeyPressed(LAMBDA_KEY_A)) {
      camera_position_.x -= camera_translation_speed_ * delta_in_ms;
    } else if (input::Input::IsKeyPressed(LAMBDA_KEY_D)) {
      camera_position_.x += camera_translation_speed_ * delta_in_ms;
    }

    camera_.SetPosition(camera_position_);

    if (can_rotate_) {
      if (input::Input::IsKeyPressed(LAMBDA_KEY_Q)) {
        camera_rotation_ -= camera_rotation_speed_  * delta_in_ms;
      } else if (input::Input::IsKeyPressed(LAMBDA_KEY_E)) {
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
    events::Event* const event) {

  events::Dispatcher::HandleWhen<events::WindowResizeEvent>(
      BIND_EVENT_HANDLER(OrthographicCameraController::OnWindowResize), event);

  events::Dispatcher::HandleWhen<events::MouseScrolledEvent>(
      BIND_EVENT_HANDLER(OrthographicCameraController::OnMouseScrolled), event);
}

}  // namespace lambda::core
