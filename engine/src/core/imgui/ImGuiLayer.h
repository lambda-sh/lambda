#ifndef ENGINE_SRC_CORE_IMGUI_IMGUILAYER_H_
#define ENGINE_SRC_CORE_IMGUI_IMGUILAYER_H_

#include "core/Core.h"
#include "core/Layer.h"
#include "core/events/ApplicationEvent.h"
#include "core/events/Event.h"
#include "core/events/KeyEvent.h"
#include "core/events/MouseEvent.h"

namespace engine {
namespace imgui {

class ENGINE_API ImGuiLayer : public Layer {
 public:
  ImGuiLayer();
  ~ImGuiLayer();

  void OnAttach() override;
  void OnDetach() override;
  void OnUpdate() override;
  void OnEvent(events::Event* event) override;
 private:
  float time_ = 0.0f;

  bool OnMouseButtonPressedEvent(const events::MouseButtonPressedEvent& event);
  bool OnMouseButtonReleasedEvent(
      const events::MouseButtonReleasedEvent& event);
  bool OnMouseMovedEvent(const events::MouseMovedEvent& event);
  bool OnMouseScrolledEvent(const events::MouseScrolledEvent& event);
  bool OnKeyPressedEvent(const events::KeyPressedEvent& event);
  bool OnKeyReleasedEvent(const events::KeyReleasedEvent& event);
  bool OnKeyTypedEvent(const events::KeyTypedEvent& event);
  bool OnWindowResizeEvent(const events::WindowResizeEvent& event);
};

}  // namespace imgui
}  // namespace engine

#endif  // ENGINE_SRC_CORE_IMGUI_IMGUILAYER_H_
