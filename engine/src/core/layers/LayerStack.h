/**
 * @file engine/src/core/LayerStack.h
 * @brief The LayerStack Definition for handling multiple layers.
 *
 * Primarily used within engine/src/core/Application.h
 */
#ifndef ENGINE_SRC_CORE_LAYERS_LAYERSTACK_H_
#define ENGINE_SRC_CORE_LAYERS_LAYERSTACK_H_

#include <vector>

#include "core/Core.h"
#include "core/layers/Layer.h"
#include "core/memory/Pointers.h"

namespace engine {
namespace layers {

/**
 * @class LayerStack
 * @brief A stack based data structure for the storage of layers to be
 * managed by the engine.
 *
 * The layer stack is completely managed by the engine. However, the engine does
 * expose functionality to safely interact with the one that is being used for
 * any given application being powered by the engine.
 */
class LayerStack {
 public:
  LayerStack();
  ~LayerStack();

  /**
   * @fn PushLayer
   * @brief Push a layer on to the Layer stack.
   */
  void PushLayer(memory::Shared<Layer> layer);

  /**
   * @fn PushOverlay
   * @brief Pushes an overlay on to the back of the stack.
   */
  void PushOverlay(memory::Shared<Layer> overlay);

  /**
   * @fn PopLayer
   * @brief Pops a layer off the layer stack.
   */
  void PopLayer(memory::Shared<Layer> layer);

  /**
   * @fn PopOverlay
   * @brief Pops an overlay off the layer stack.
   */
  void PopOverlay(memory::Shared<Layer> layer);

  std::vector<memory::Shared<Layer>>::iterator begin() {
    return layers_.begin(); }
  std::vector<memory::Shared<Layer>>::iterator end() { return layers_.end(); }

  std::vector<memory::Shared<Layer>>::reverse_iterator rbegin() {
    return layers_.rbegin(); }
  std::vector<memory::Shared<Layer>>::reverse_iterator rend() {
    return layers_.rend(); }

 private:
  std::vector<memory::Shared<Layer>> layers_;
  unsigned int layer_insert_location_ = 0;
};

}  // namespace layers
}  // namespace engine

#endif  // ENGINE_SRC_CORE_LAYERS_LAYERSTACK_H_
