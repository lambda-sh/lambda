#include <Lambda/core/layers/LayerStack.h>

#include <vector>

#include <Lambda/core/layers/Layer.h>
#include <Lambda/core/memory/Pointers.h>

namespace lambda::core::layers {

LayerStack::LayerStack() = default;

/// Does one final detach on all of the layers when being closed out. This is
/// for allowing all of the layers attached to the application to gracefully
/// detach one more time before the application completes its shutdown.
///
/// @todo Should layers be gracefully handled or should it be up to the user to
/// remove the layers before shutdown?
LayerStack::~LayerStack() {
  for (const auto& layer : layers_) {
    layer->OnDetach();
  }
}

void LayerStack::PushLayer(memory::Shared<Layer> layer) {
  layers_.emplace(layers_.begin() + layer_insert_location_, layer);
  ++layer_insert_location_;
}

void LayerStack::PushOverlay(memory::Shared<Layer> overlay) {
  layers_.emplace_back(overlay);
}

/// Pop a layer off of the layer stack. Compares layers via Shared resources.
/// TODO(C3NZ): Is this problematic since it's using a shared pointer?
void LayerStack::PopLayer(memory::Shared<Layer> layer) {
  if (const auto it = std::find(layers_.begin(), layers_.end(), layer);
      it != layers_.end()) {
    layers_.erase(it);
    --layer_insert_location_;
  }
}

/// TODO(C3NZ): Is this problematic since it's using a shared pointer?
void LayerStack::PopOverlay(memory::Shared<Layer> overlay) {
  if (auto it = std::find(layers_.begin(), layers_.end(), overlay);
      it != layers_.end()) {
    layers_.erase(it);
  }
}

}  // namespace lambda::core::layers
