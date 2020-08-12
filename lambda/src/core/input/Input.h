/**
 * @file Input.h
 * @brief The input abstraction class that handles input across
 */
#ifndef LAMBDA_SRC_CORE_INPUT_INPUT_H_
#define LAMBDA_SRC_CORE_INPUT_INPUT_H_

#include <utility>

#include "core/Core.h"

namespace lambda {
namespace core {
namespace input {

class Input {
 public:
  static bool IsKeyPressed(int key_code) {
      return kInput_->IsKeyPressedImpl(key_code); }

  static float GetMouseX() { return kInput_->GetMouseXImpl(); }
  static float GetMouseY() { return kInput_->GetMouseYImpl(); }

  static std::pair<float, float> GetMousePosition() {
      return kInput_->GetMousePositionImpl(); }

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

/**
 * @class lambda::Input
 * @brief The generalized Input class for all input systems.
 *
 * All Input instances will be child classes of Input, but never called directly
 * in order to abstract platform specific implementations.
 */


/**
 * @function lambda::Input::IsKeyPressed
 * @brief Check if the current key is being pressed.
 */


/**
 * @function lambda::Input::GetMouseX
 * @brief Get the current mouse x position.
 */

/**
 * @function lambda::Input::GetMouseY
 * @brief Get the current mouse y position.
 */

/**
 * @function lambda::Input::GetMousePosition
 * @brief Get the current mouse x & y positions.
 */

/**
 * @function lambda::Input::IsMouseButtonPressed
 * @brief Check to see if a mouse button is being pressed.
 */
