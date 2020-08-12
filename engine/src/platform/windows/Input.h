#ifndef ENGINE_SRC_PLATFORM_WINDOWS_INPUT_H_
#define ENGINE_SRC_PLATFORM_WINDOWS_INPUT_H_

#if defined ENGINE_PLATFORM_WINDOWS || defined ENGINE_DEBUG

#include <utility>

#include "core/input/Input.h"

namespace engine {
namespace platform {
namespace windows {

/// @brief The windows input implementation.
// TODO(C3NZ): Rename this to WindowsInput to create a class system for all
// windows platform specific classes.
class InputImplementation : public core::input::Input {
 protected:
  bool IsKeyPressedImpl(int key_code) override;

  float GetMouseXImpl() override;
  float GetMouseYImpl() override;
  std::pair<float, float> GetMousePositionImpl() override;
  bool IsMouseButtonPressedImpl(int button) override;
};

}  // namespace windows
}  // namespace platform
}  // namespace engine

#endif  // ENGINE_PLATFORM_WINDOWS
#endif  // ENGINE_SRC_PLATFORM_WINDOWS_INPUT_H_
