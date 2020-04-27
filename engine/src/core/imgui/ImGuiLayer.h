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

// ImguiLayer base implementation.
class ENGINE_API ImGuiLayer : public Layer {
 public:
  ImGuiLayer();
  ~ImGuiLayer();

  void OnAttach() override;
  void OnDetach() override;

  void OnImGuiRender();
  void Begin();
  void End();
 private:
  float time_ = 0.0f;
};

}  // namespace imgui
}  // namespace engine

#endif  // ENGINE_SRC_CORE_IMGUI_IMGUILAYER_H_
