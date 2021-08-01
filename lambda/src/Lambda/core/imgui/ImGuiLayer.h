/// @file ImGuiLayer.h
/// @brief The ImGuiLayer implementation for dev tool creation.
///
/// Any application that inherits from the game engine should not compile with
/// imgui. It is very performance heavy and will cause your application to
/// perform magnitudes slower.
#ifndef LAMBDA_SRC_LAMBDA_CORE_IMGUI_IMGUILAYER_H_
#define LAMBDA_SRC_LAMBDA_CORE_IMGUI_IMGUILAYER_H_

#include <Lambda/core/Core.h>
#include <Lambda/core/events/ApplicationEvent.h>
#include <Lambda/core/events/Event.h>
#include <Lambda/core/events/KeyEvent.h>
#include <Lambda/core/events/MouseEvent.h>
#include <Lambda/core/layers/Layer.h>

namespace lambda::core::imgui {

/// @brief The base ImGui layer used for rendering all other ImGui
/// components.
class ImGuiLayer : public layers::Layer {
 public:
  ImGuiLayer();
  ~ImGuiLayer() override;

  /// @brief What to do when attached to lambda.
  void OnAttach() override;
  /// @brief What to do when detached from lambda.
  void OnDetach() override;
  /// @brief What to do when ImGui requests to render.
  void OnImGuiRender() override;

  void OnUpdate(lib::TimeStep time_step) override {}

  /// @brief Begin an ImGui rendering context.
  void Begin();
  /// @brief End an ImGui rendering context.
  void End();

  void OnEvent(events::Event* const event) override {}
 private:
  float time_ = 0.0f;
  static bool show_demo_window_;
};

}  // namespace lambda::core::imgui

#endif  // LAMBDA_SRC_LAMBDA_CORE_IMGUI_IMGUILAYER_H_
