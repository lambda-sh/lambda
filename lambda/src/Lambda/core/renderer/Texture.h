/// @file Texture.h
/// @brief The Abstract Texture implementation
#ifndef LAMBDA_SRC_LAMBDA_CORE_RENDERER_TEXTURE_H_
#define LAMBDA_SRC_LAMBDA_CORE_RENDERER_TEXTURE_H_

#include <bits/stdint-uintn.h>
#include <string>

#include "Lambda/core/memory/Pointers.h"

namespace lambda {
namespace core {
namespace renderer {

/// @brief The abstract texture API.
///
/// Implemented by platform specific APIs
class Texture {
 public:
  virtual ~Texture() = default;

  /// @brief Get the width of the texture.
  virtual uint32_t GetWidth() const = 0;
  /// @brief Get the height of the texture.
  virtual uint32_t GetHeight() const = 0;

  /// @brief Bind the texture to the GPU.
  virtual void Bind(uint32_t slot = 0) const = 0;
};

/// @brief The 2D Texture API.
///
/// Currently just a wrapper around the Texture API.
class Texture2D : public Texture {
 public:
  /// @brief Create a 2D Texture given the path to a texture asset.
  static memory::Shared<Texture2D> Create(const std::string& path);
};

}  // namespace renderer
}  // namespace core
}  // namespace lambda

#endif  // LAMBDA_SRC_LAMBDA_CORE_RENDERER_TEXTURE_H_
