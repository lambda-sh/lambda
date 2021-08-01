/// @file Layer.h
/// @brief The Layer implementation that allows application to specify layers to
/// be attached to the game.
#ifndef LAMBDA_SRC_LAMBDA_CORE_LAYERS_LAYER_H_
#define LAMBDA_SRC_LAMBDA_CORE_LAYERS_LAYER_H_

#include <Lambda/core/events/Event.h>
#include <Lambda/core/memory/Pointers.h>
#include <Lambda/lib/Time.h>

namespace lambda::core::layers {

/// @brief The lambda Layer abstraction. Primarily used for direct access into
/// lambdas tick and event system.
///
/// Anyone implementing lambda as a library in their project will at some point
/// need to create a layer in order to receive updates and events from lambda.
class Layer {
 public:
  explicit Layer(std::string name = "Layer") : debug_name_(std::move(name)) {}
  virtual ~Layer() = default;

  /// @brief What to do when a layer is attached to lambda.
  virtual void OnAttach() = 0;

  /// @brief What to do when a layer is detached from lambda.
  virtual void OnDetach() = 0;

  /// @brief What to do when lambda has an update for the layer.
  virtual void OnUpdate(lib::TimeStep time_step) = 0;

  /// @brief What to do when lambda has an event for the layer.
  virtual void OnEvent(events::Event* const event) = 0;

  /// @brief What to do when lambda is rendering with ImGui.
  virtual void OnImGuiRender() = 0;

  /// @brief Get the debug name of the layer. Might not be in production builds.
  [[nodiscard]] const std::string& GetName() const { return debug_name_; }

 protected:
  std::string debug_name_;
};

}  // namespace lambda::core::layers

#endif  // LAMBDA_SRC_LAMBDA_CORE_LAYERS_LAYER_H_
