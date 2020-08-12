#include "core/layers/Layer.h"

#include <string>

namespace engine {
namespace core {
namespace layers {

Layer::Layer(const std::string& debug_name) : debug_name_(debug_name) {}

Layer::~Layer() {}

}  // namespace layers
}  // namespace core
}  // namespace engine
