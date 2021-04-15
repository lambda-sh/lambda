/// @file Input.h
/// @brief The input abstraction class that handles input across
#ifndef LAMBDA_SRC_LAMBDA_CORE_INPUT_INPUT_H_
#define LAMBDA_SRC_LAMBDA_CORE_INPUT_INPUT_H_

#include <utility>

#include <Lambda/core/memory/Pointers.h>

namespace lambda::core::input {

/// @brief The generic input system for getting input data from
/// applications built on lambda.
class Input {
 public:
  /// @brief Check if a lambda key code was pressed.
  [[nodiscard]] static bool IsKeyPressed(const int key_code) {
    return kInput_->IsKeyPressedImpl(key_code);
  }

  /// @brief Get the current Mouse X Coordinate.
  [[nodiscard]] static float GetMouseX() {
    return kInput_->GetMouseXImpl();
  }
  /// @brief Get the current Mouse Y Coordinate.
  [[nodiscard]] static float GetMouseY() {
    return kInput_->GetMouseYImpl();
  }

  /// @brief Get the current mouse position (X, Y) as a pair.
  [[nodiscard]] static std::pair<float, float> GetMousePosition() {
      return kInput_->GetMousePositionImpl();
  }

  /// @brief Check to see if a mouse button is being pressed.
  [[nodiscard]] static bool IsMouseButtonPressed(const int button) {
      return kInput_->IsMouseButtonPressedImpl(button);
  }

 protected:
  [[nodiscard]] virtual bool IsKeyPressedImpl(int key_code) = 0;

  [[nodiscard]] virtual float GetMouseXImpl() = 0;
  [[nodiscard]] virtual float GetMouseYImpl() = 0;
  [[nodiscard]] virtual std::pair<float, float> GetMousePositionImpl() = 0;
  [[nodiscard]] virtual bool IsMouseButtonPressedImpl(int button) = 0;

 private:
  static memory::Unique<Input> kInput_;
};

}  // namespace lambda::core::input

#endif  // LAMBDA_SRC_LAMBDA_CORE_INPUT_INPUT_H_
