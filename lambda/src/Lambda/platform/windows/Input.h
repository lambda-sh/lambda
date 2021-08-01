#ifndef LAMBDA_SRC_LAMBDA_PLATFORM_WINDOWS_INPUT_H_
#define LAMBDA_SRC_LAMBDA_PLATFORM_WINDOWS_INPUT_H_

#if defined LAMBDA_PLATFORM_WINDOWS || defined LAMBDA_DEBUG

#include <utility>

#include <Lambda/core/input/Input.h>

namespace lambda::platform::windows {

/// @brief The windows input implementation.
/// TODO(C3NZ): Rename this to WindowsInput to create a class system for all
// windows platform specific classes.
class InputImplementation : public core::input::Input {
 protected:
  bool IsKeyPressedImpl(int key_code) override;

  float GetMouseXImpl() override;
  float GetMouseYImpl() override;
  std::pair<float, float> GetMousePositionImpl() override;
  bool IsMouseButtonPressedImpl(int button) override;
};

}  // namespace lambda::platform::windows

#endif  // LAMBDA_PLATFORM_WINDOWS
#endif  // LAMBDA_SRC_LAMBDA_PLATFORM_WINDOWS_INPUT_H_
