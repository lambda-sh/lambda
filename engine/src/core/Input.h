#ifndef ENGINE_SRC_CORE_INPUT_H_
#define ENGINE_SRC_CORE_INPUT_H_

#include <utility>

#include "core/Core.h"

namespace engine {

// The generalized Input class for all input systems. All Input instances will
// be child classes of Input, but never called directly in order to abstract
// platform specific implementations.
class ENGINE_API Input {
 public:
  // -------------------------------- Key input --------------------------------

  inline static bool IsKeyPressed(int key_code)
      { return kInput_->IsKeyPressedImpl(key_code); }

  // ------------------------------- Mouse input -------------------------------

  inline static float GetMouseX() { return kInput_->GetMouseXImpl(); }
  inline static float GetMouseY() { return kInput_->GetMouseYImpl(); }
  inline static std::pair<float, float> GetMousePosition() {
      return kInput_->GetMousePositionImpl(); }
  inline static bool IsMouseButtonPressed(int button) {
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

}  // namespace engine

#endif  // ENGINE_SRC_CORE_INPUT_H_
