#ifndef ENGINE_SRC_CORE_LAYER_H_
#define ENGINE_SRC_CORE_LAYER_H_

#include "core/Core.h"
#include "core/events/Event.h"

namespace engine {

class ENGINE_API Layer {
 public:
  explicit Layer(const std::string& name = "Layer");
  virtual ~Layer();

  virtual void OnAttach() {}
  virtual void OnDetach() {}
  virtual void OnUpdate() {}
  virtual void OnEvent(events::Event* event) {}

  inline const std::string& GetName() const { return debug_name_; }
 protected:
  std::string debug_name_;
};

}  // namespace engine

#endif  // ENGINE_SRC_CORE_LAYER_H_
