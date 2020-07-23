/**
 * @file ImGuiLayer.h
 * @brief The ImGuiLayer implementation for dev tool creation.
 *
 * Any application that inherits from the game engine should not compile with
 * imgui. It is very performance heavy and will cause your application to
 * perform magnitudes slower.
 */
#ifndef ENGINE_SRC_CORE_IMGUI_IMGUILAYER_H_
#define ENGINE_SRC_CORE_IMGUI_IMGUILAYER_H_

#include "core/Core.h"
#include "core/layers/Layer.h"
#include "core/events/ApplicationEvent.h"
#include "core/events/Event.h"
#include "core/events/KeyEvent.h"
#include "core/events/MouseEvent.h"

namespace engine {
namespace imgui {

class ImGuiLayer : public layers::Layer {
 public:
  ImGuiLayer();
  ~ImGuiLayer();

  void OnAttach() override;
  void OnDetach() override;
  void OnImGuiRender() override;

  void Begin();
  void End();

 private:
  float time_ = 0.0f;
  static bool show_demo_window_;
};

}  // namespace imgui
}  // namespace engine

#endif  // ENGINE_SRC_CORE_IMGUI_IMGUILAYER_H_

/**
 * @class engine::imgui::ImGuiLayer
 * @brief An abstract Imgui layer implementation
 */

/**
 * @fn engine::imgui::ImGuiLayer::OnAttach
 * @brief Handles an ImGuiLayers attachment to the engine.
 *
 * This is currently setup with a default implementation but will most likely
 * be delegated to users that would like to implement their own imgui layers.
 */

/**
 * @fn engine::imgui::ImGuiLayer::OnDetach
 * @brief Handles an ImGuiLayers detachment to the engine.
 *
 * This is currently setup with a default implementation but will most likely
 * be delegated to users that would like to implement their own imgui layers.
 */

/**
 * @fn engine::imgui::ImGuiLayer::OnImGuiRender
 * @brief Handles an ImGuiRender Call by the engine.
 *
 * This will only be called when the engine is compiled with imgui attached.
 */

/**
 * @fn engine::imgui::ImGuiLayer::Begin
 * @brief Instantiates the context for the Imgui layer.
 *
 * Must be closed out with End() in order to prevent the imgui context from
 * leaking and interfering with other graphics trying to be rendered.
 */

/**
 * @fn engine::imgui::ImGuiLayer::End
 * @brief Closes and cleans up the context for the Imgui layer.
 *
 * Must be opened with Begin() in order to prevent the imgui context from
 * leaking and interfering with other graphics trying to be rendered.
 */
