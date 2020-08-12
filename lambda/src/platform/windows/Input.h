#ifndef LAMBDA_SRC_PLATFORM_WINDOWS_INPUT_H_
#define LAMBDA_SRC_PLATFORM_WINDOWS_INPUT_H_

#if defined LAMBDA_PLATFORM_WINDOWS || defined LAMBDA_DEBUG

#include <utility>

#include "core/input/Input.h"

namespace lambda {
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
}  // namespace lambda

#endif  // LAMBDA_PLATFORM_WINDOWS
#endif  // LAMBDA_SRC_PLATFORM_WINDOWS_INPUT_H_
