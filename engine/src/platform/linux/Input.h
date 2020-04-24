#ifndef ENGINE_SRC_PLATFORM_LINUX_INPUT_H_
#define ENGINE_SRC_PLATFORM_LINUX_INPUT_H_

#if defined ENGINE_PLATFORM_LINUX || defined ENGINE_DEBUG

#include <utility>

#include <GLFW/glfw3.h>

#include "core/Input.h"

// TODO(C3NZ): Implement this for the windows platform
namespace engine {
namespace platform {
namespace linux {

class InputImplementation : public Input {
 protected:
  bool IsKeyPressedImpl(int key_code) override;

  float GetMouseXImpl() override;
  float GetMouseYImpl() override;
  std::pair<float, float> GetMousePositionImpl() override;
  bool IsMouseButtonPressedImpl(int button) override;
};

}  // namespace linux
}  // namespace platform
}  // namespace engine

#endif  // ENGINE_PLATFORM_LINUX
#endif  // ENGINE_SRC_PLATFORM_LINUX_INPUT_H_
