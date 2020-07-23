/**
 * @file Layer.h
 * @brief The Layer implementation that allows application to specify layers to
 * be attached to the game.
 */
#ifndef ENGINE_SRC_CORE_LAYERS_LAYER_H_
#define ENGINE_SRC_CORE_LAYERS_LAYER_H_

#include "core/Core.h"
#include "core/events/Event.h"
#include "core/util/Time.h"

namespace engine {
namespace layers {

class Layer {
 public:
  explicit Layer(const std::string& name = "Layer");
  virtual ~Layer();

  virtual void OnAttach() {}
  virtual void OnDetach() {}
  virtual void OnUpdate(util::TimeStep time_step) {}
  virtual void OnEvent(events::Event* event) {}
  virtual void OnImGuiRender() {}

  inline const std::string& GetName() const { return debug_name_; }

 protected:
  std::string debug_name_;
};


}  // namespace layers
}  // namespace engine

#endif  // ENGINE_SRC_CORE_LAYERS_LAYER_H_

/**
 * @class engine::layers::Layer
 * @brief An abstract data structure that represents a "layer" within the
 * engine.
 *
 * Layers are the primary hook for applications to specify logic that they want
 * to be apart of the engines event loop.
 */

/**
 * @fn engine::layers::Layer::OnAttach
 * @brief What to do when a layer is attached to the game engine.
 *
 * Primarily for initializing anything in the layer when it's attached to the
 * engine.
 */

/**
 * @fn engine::layers::Layer::OnDetach
 * @brief Handles what to do when a layer is attached to the game engine.
 *
 * Primarily for cleaning up anything in the layer when it's no longer
 * attached to the engine.
 */

/**
 * @fn engine::layers::Layer::OnUpdate
 * @brief Handles what to do when the game engine requests to update the
 * layer.
 *
 * This can only be accessed if the layer is attached to the engine.
 */

/**
 * @fn engine::layers::Layer::OnEvent
 * @param event - The event received by the engine.
 * @brief Handles what to do when the game engine passes an event.
 *
 * This can only be accessed if the layer is attached to the engine.
 */

/**
 * @fn engine::layers::Layer::OnImGuiRender
 * @brief Handles what to do when the game engine requests the layer to
 * render ImGui components.
 *
 * This can only be accessed if the layer is attached to the engine.
 */
