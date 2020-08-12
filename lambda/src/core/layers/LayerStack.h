/// @file LayerStack.h
/// @brief The LayerStack Definition for handling multiple layers.
///
/// Primarily used within engine/src/core/Application.h
#ifndef LAMBDA_SRC_CORE_LAYERS_LAYERSTACK_H_
#define LAMBDA_SRC_CORE_LAYERS_LAYERSTACK_H_

#include <vector>

#include "core/Core.h"
#include "core/layers/Layer.h"
#include "core/memory/Pointers.h"

namespace lambda {
namespace core {
namespace layers {

/// @brief A Stack based structure for lambda to manage layers in.
///
/// Application primarily uses this internally to allow developers to use lambda
/// as the manager for any layers that they create.
class LayerStack {
 public:
  LayerStack();
  ~LayerStack();

  /// @brief Push a layer into the layer stack.
  void PushLayer(memory::Shared<Layer> layer);
  /// @brief Push an overlay into the layer stack.
  void PushOverlay(memory::Shared<Layer> overlay);
  /// @brief Pop a layer out of the layer stack.
  void PopLayer(memory::Shared<Layer> layer);
  /// @brief Pop an overlay out of the layer stack.
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
}  // namespace core
}  // namespace lambda

#endif  // LAMBDA_SRC_CORE_LAYERS_LAYERSTACK_H_
