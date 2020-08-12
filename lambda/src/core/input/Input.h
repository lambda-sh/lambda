/// @file Input.h
/// @brief The input abstraction class that handles input across
#ifndef LAMBDA_SRC_CORE_INPUT_INPUT_H_
#define LAMBDA_SRC_CORE_INPUT_INPUT_H_

#include <utility>

#include "core/Core.h"

namespace lambda {
namespace core {
namespace input {

/// @brief The generic input system for getting input data from
/// applications built on lambda.
class Input {
 public:
  /// @brief Check if a lambda key code was pressed.
  static bool IsKeyPressed(int key_code) {
      return kInput_->IsKeyPressedImpl(key_code); }

  /// @brief Get the current Mouse X Coordinate.
  static float GetMouseX() { return kInput_->GetMouseXImpl(); }
  /// @brief Get the current Mouse Y Coordinate.
  static float GetMouseY() { return kInput_->GetMouseYImpl(); }

  /// @brief Get the current mouse position (X, Y) as a pair.
  static std::pair<float, float> GetMousePosition() {
      return kInput_->GetMousePositionImpl(); }

  /// @brief Check to see if a mouse button is being pressed.
  static bool IsMouseButtonPressed(int button) {
      return kInput_->IsMouseButtonPressedImpl(button); }

 protected:
  virtual bool IsKeyPressedImpl(int key_code) = 0;

  virtual float GetMouseXImpl() = 0;
  virtual float GetMouseYImpl() = 0;
  virtual std::pair<float, float> GetMousePositionImpl() = 0;
  virtual bool IsMouseButtonPressedImpl(int button) = 0;

 private:
  static Input* kInput_;
};

}  // namespace input
}  // namespace core
}  // namespace lambda

#endif  // LAMBDA_SRC_CORE_INPUT_INPUT_H_
