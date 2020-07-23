#include "core/Layer.h"

#include <string>

namespace engine {

Layer::Layer(const std::string& debug_name) : debug_name_(debug_name) {}

Layer::~Layer() {}

}  // namespace engine
