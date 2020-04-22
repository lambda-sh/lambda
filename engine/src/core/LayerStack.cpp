#include "core/LayerStack.h"

#include <vector>

#include "core/Layer.h"

namespace engine {

LayerStack::LayerStack() {
  layer_insert_location_ = layers_.begin();
}

// Layers are destroyed as soon as the layer stack also is.
LayerStack::~LayerStack() {
  for (Layer* layer : layers_) {
    delete layer;
  }
}

void LayerStack::PushLayer(Layer* layer) {
  layer_insert_location_ = layers_.emplace(layer_insert_location_, layer);
}

// Layers will always be pushed into the back of the list as the last thing to
// be rendered/handled. This ensures that overlays are always on top of layers
void LayerStack::PushOverlay(Layer* overlay) {
  layers_.emplace_back(overlay);
}

// Once a layer has been popped off of the LayerStack, it is no longer managed
// by the layer stack.
void LayerStack::PopLayer(Layer* layer) {
  auto it = std::find(layers_.begin(), layers_.end(), layer);
  if (it != layers_.end()) {
    layers_.erase(it);
    layer_insert_location_--;
  }
}

// Once an overlay has been popped off of the LayerStack, it is no longer
// managed by the layer stack.
void LayerStack::PopOverlay(Layer* overlay) {
  auto it = std::find(layers_.begin(), layers_.end(), overlay);
  if (it != layers_.end()) {
    layers_.erase(it);
  }
}

}  // namespace engine
