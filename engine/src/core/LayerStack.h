#ifndef ENGINE_SRC_CORE_LAYERSTACK_H_
#define ENGINE_SRC_CORE_LAYERSTACK_H_

#include <vector>

#include "core/Core.h"
#include "core/Layer.h"

namespace engine {

class ENGINE_API LayerStack {
 public:
  LayerStack();
  ~LayerStack();

  // Push a layer on to the Layer stack.
  void PushLayer(Layer* layer);
  // Push an overlay on the layer stack.
  void PushOverlay(Layer* overlay);
  // Pop a layer off of the Layer stack.
  void PopLayer(Layer* layer);
  // Pop an overlay off of to the Layer stack.
  void PopOverlay(Layer* layer);

  // Get the beginning of the layer stack.
  std::vector<Layer*>::iterator begin() { return layers_.begin(); }
  // Get the end of the layer stack.
  std::vector<Layer*>::iterator end() { return layers_.end(); }
 private:
  std::vector<Layer*> layers_;
  unsigned int layer_insert_location_ = 0;
};

}  // namespace engine

#endif  // ENGINE_SRC_CORE_LAYERSTACK_H_
