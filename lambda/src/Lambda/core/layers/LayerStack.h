/// @file LayerStack.h
/// @brief The LayerStack Definition for handling multiple layers.
///
/// Primarily used within engine/src/core/Application.h
#ifndef LAMBDA_SRC_LAMBDA_CORE_LAYERS_LAYERSTACK_H_
#define LAMBDA_SRC_LAMBDA_CORE_LAYERS_LAYERSTACK_H_

#include <vector>

#include <Lambda/core/layers/Layer.h>
#include <Lambda/core/memory/Pointers.h>

namespace lambda::core::layers {

/// @brief A Stack based structure for lambda to manage layers in.
///
/// Application primarily uses this internally to allow developers to use lambda
/// as the manager for any layers that they create.
class LayerStack {
  typedef std::vector<memory::Shared<Layer>> LayerContainer;
  typedef LayerContainer::iterator Iterator;
  typedef LayerContainer::reverse_iterator ReverseIterator;

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
  void PopOverlay(memory::Shared<Layer> overlay);

  Iterator begin() {
    return layers_.begin();
  }

  Iterator end() {
    return layers_.end();
  }

  ReverseIterator rbegin() {
    return layers_.rbegin();
  }

  ReverseIterator rend() {
    return layers_.rend();
  }

 private:
  LayerContainer layers_;
  unsigned int layer_insert_location_ = 0;
};

}  // namespace lambda::core::layers

#endif  // LAMBDA_SRC_LAMBDA_CORE_LAYERS_LAYERSTACK_H_
