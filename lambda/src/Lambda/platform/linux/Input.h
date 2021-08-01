#ifndef LAMBDA_SRC_LAMBDA_PLATFORM_LINUX_INPUT_H_
#define LAMBDA_SRC_LAMBDA_PLATFORM_LINUX_INPUT_H_

#if defined LAMBDA_PLATFORM_LINUX || defined LAMBDA_DEBUG

#include <utility>

#include <Lambda/core/input/Input.h>

/// TODO(C3NZ): Implement this for the windows platform
namespace lambda::platform::linux {

class InputImplementation : public core::input::Input {
 protected:
  bool IsKeyPressedImpl(int key_code) override;

  float GetMouseXImpl() override;
  float GetMouseYImpl() override;
  std::pair<float, float> GetMousePositionImpl() override;
  bool IsMouseButtonPressedImpl(int button) override;
};

}  // namespace lambda::platform::linux

#endif  // LAMBDA_PLATFORM_LINUX
#endif  // LAMBDA_SRC_LAMBDA_PLATFORM_LINUX_INPUT_H_
