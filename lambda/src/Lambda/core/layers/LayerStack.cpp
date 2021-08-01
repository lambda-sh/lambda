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
  for (auto& layer : layers_) {
    layer->OnDetach();
  }
}

void LayerStack::PushLayer(memory::Unique<Layer> layer) {
  layers_.emplace(layers_.begin() + layer_insert_location_, std::move(layer));
  ++layer_insert_location_;
}

void LayerStack::PushOverlay(memory::Unique<Layer> overlay) {
  layers_.emplace_back(std::move(overlay));
}

void LayerStack::RemoveLayer(std::string_view layer_name) {
  const auto result = std::find_if(
      layers_.begin(), 
      layers_.end(), 
      [&](auto& layer) {
        return layer_name == layer->GetName();
      });

  [[likely]] if (result != layers_.end()) {
    layers_.erase(result);
    --layer_insert_location_;
  }
}

void LayerStack::RemoveOverlay(std::string_view overlay_name) {
  const auto result = std::find_if(
      layers_.begin(), 
      layers_.end(), 
      [&](auto& layer) {
        return overlay_name == layer->GetName();
      });

  [[likely]] if (result != layers_.end()) {
    layers_.erase(result);
  }
}

}  // namespace lambda::core::layers
