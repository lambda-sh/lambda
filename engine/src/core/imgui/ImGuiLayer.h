#ifndef ENGINE_SRC_CORE_IMGUI_IMGUILAYER_H_
#define ENGINE_SRC_CORE_IMGUI_IMGUILAYER_H_

#include "core/Core.h"
#include "core/Layer.h"
#include "core/events/Event.h"

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
  float time_;
};


}  // namespace imgui
}  // namespace engine

#endif  // ENGINE_SRC_CORE_IMGUI_IMGUILAYER_H_
