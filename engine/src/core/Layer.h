/**
 * @file engine/src/core/Layer.h
 * @brief The Layer implementation that allows application to specify layers to
 * be attached to the game.
 *
 * Layers are the primary hook for applications to specify logic that they want
 * to be apart of the engines event loop.
 */
#ifndef ENGINE_SRC_CORE_LAYER_H_
#define ENGINE_SRC_CORE_LAYER_H_

#include "core/Core.h"
#include "core/events/Event.h"
#include "core/util/Time.h"

namespace engine {

class ENGINE_API Layer {
 public:
  explicit Layer(const std::string& name = "Layer");
  virtual ~Layer();

  /**
   * @fn OnAttach
   * @brief What to do when a layer is attached to the game engine.
   *
   * Primarily for initializing anything in the layer when it's attached to the
   * engine.
   */
  virtual void OnAttach() {}

  /**
   * @fn OnDetach
   * @brief Handles what to do when a layer is attached to the game engine.
   *
   * Primarily for cleaning up anything in the layer when it's no longer
   * attached to the engine.
   */
  virtual void OnDetach() {}

  /**
   * @fn OnUpdate
   * @brief Handles what to do when the game engine requests to update the
   * layer.
   *
   * This can only be accessed if the layer is attached to the engine.
   */
  virtual void OnUpdate(util::TimeStep time_step) {}

  /**
   * @fn OnEvent
   * @param event - The event received by the engine.
   * @brief Handles what to do when the game engine passes an event.
   *
   * This can only be accessed if the layer is attached to the engine.
   */
  virtual void OnEvent(events::Event* event) {}

  /**
   * @fn OnImGuiRender
   * @brief Handles what to do when the game engine requests the layer to
   * render ImGui components.
   *
   * This can only be accessed if the layer is attached to the engine.
   */
  virtual void OnImGuiRender() {}

  /**
   * @fn GetName
   * @brief Gets the name of the layer.
   *
   * Mainly used for debugging within the engine.
   */
  inline const std::string& GetName() const { return debug_name_; }

 protected:
  std::string debug_name_;
};

}  // namespace engine

#endif  // ENGINE_SRC_CORE_LAYER_H_
