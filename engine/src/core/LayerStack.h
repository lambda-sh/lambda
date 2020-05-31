/**
 * @file engine/src/core/LayerStack.h
 * @brief The LayerStack Definition for handling multiple layers.
 *
 * Primarily used within engine/src/core/Application.h
 */
#ifndef ENGINE_SRC_CORE_LAYERSTACK_H_
#define ENGINE_SRC_CORE_LAYERSTACK_H_

#include <vector>

#include "core/Core.h"
#include "core/Layer.h"

namespace engine {

/**
 * @class LayerStack
 * @brief A stack based data structure for the storage of layers to be
 * managed by the engine.
 *
 * The layer stack is completely managed by the engine. However, the engine does
 * expose functionality to safely interact with the one that is being used for
 * any given application being powered by the engine.
 */
class ENGINE_API LayerStack {
 public:
  LayerStack();
  ~LayerStack();

  /**
   * @fn PushLayer
   * @brief Push a layer on to the Layer stack.
   */
  void PushLayer(Layer* layer);

  /**
   * @fn PushOverlay
   * @brief Pushes an overlay on to the back of the stack.
   */
  void PushOverlay(Layer* overlay);

  /**
   * @fn PopLayer
   * @brief Pops a layer off the layer stack.
   */
  void PopLayer(Layer* layer);

  /**
   * @fn PopOverlay
   * @brief Pops an overlay off the layer stack.
   */
  void PopOverlay(Layer* layer);

  std::vector<Layer*>::iterator begin() { return layers_.begin(); }
  std::vector<Layer*>::iterator end() { return layers_.end(); }

 private:
  std::vector<Layer*> layers_;
  unsigned int layer_insert_location_ = 0;
};

}  // namespace engine

#endif  // ENGINE_SRC_CORE_LAYERSTACK_H_
