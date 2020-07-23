#include "core/layers/LayerStack.h"

#include <vector>

#include "core/layers/Layer.h"
#include "core/memory/Pointers.h"

namespace engine {
namespace layers {

LayerStack::LayerStack() {}

LayerStack::~LayerStack() {}

void LayerStack::PushLayer(memory::Shared<Layer> layer) {
  layers_.emplace(layers_.begin() + layer_insert_location_, layer);
  ++layer_insert_location_;
}

void LayerStack::PushOverlay(memory::Shared<Layer> overlay) {
  layers_.emplace_back(overlay);
}

void LayerStack::PopLayer(memory::Shared<Layer> layer) {
  auto it = std::find(layers_.begin(), layers_.end(), layer);
  if (it != layers_.end()) {
    layers_.erase(it);
    --layer_insert_location_;
  }
}

void LayerStack::PopOverlay(memory::Shared<Layer> overlay) {
  auto it = std::find(layers_.begin(), layers_.end(), overlay);
  if (it != layers_.end()) {
    layers_.erase(it);
  }
}

}  // namespace layers
}  // namespace engine
