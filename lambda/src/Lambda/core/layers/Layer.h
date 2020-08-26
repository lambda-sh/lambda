/// @file Layer.h
/// @brief The Layer implementation that allows application to specify layers to
/// be attached to the game.
#ifndef LAMBDA_SRC_LAMBDA_CORE_LAYERS_LAYER_H_
#define LAMBDA_SRC_LAMBDA_CORE_LAYERS_LAYER_H_

#include "core/Core.h"
#include "core/events/Event.h"
#include "core/memory/Pointers.h"
#include "core/util/Time.h"

namespace lambda {
namespace core {
namespace layers {

/// @brief The lambda Layer abstraction. Primarily used for direct access into
/// lambdas tick and event system.
///
/// Anyone implementing lambda as a library in their project will at some point
/// need to create a layer in order to receive updates and events from lambda.
class Layer {
 public:
  explicit Layer(const std::string& name = "Layer");
  virtual ~Layer();

  /// @brief What to do when a layer is attached to lambda.
  virtual void OnAttach() {}
  /// @brief What to do when a layer is detached from lambda.
  virtual void OnDetach() {}
  /// @brief What to do when lambda has an update for the layer.
  virtual void OnUpdate(util::TimeStep time_step) {}
  /// @brief What to do when lambda has an event for the layer.
  virtual void OnEvent(memory::Shared<events::Event> event) {}
  /// @brief What to do when lambda is rendering with ImGui.
  virtual void OnImGuiRender() {}

  /// @brief Get the debug name of the layer. Might not be in production builds.
  const std::string& GetName() const { return debug_name_; }

 protected:
  std::string debug_name_;
};

}  // namespace layers
}  // namespace core
}  // namespace lambda

#endif  // LAMBDA_SRC_LAMBDA_CORE_LAYERS_LAYER_H_
