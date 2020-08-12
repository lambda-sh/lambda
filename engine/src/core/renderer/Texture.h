/**
 * @file Texture.h
 * @brief The Abstract Texture implementation
 */
#ifndef ENGINE_SRC_CORE_RENDERER_TEXTURE_H_
#define ENGINE_SRC_CORE_RENDERER_TEXTURE_H_

#include <bits/stdint-uintn.h>
#include <string>

#include "core/memory/Pointers.h"

namespace engine {
namespace core {
namespace renderer {

class Texture {
 public:
  virtual ~Texture() = default;

  virtual uint32_t GetWidth() const = 0;
  virtual uint32_t GetHeight() const = 0;

  virtual void Bind(uint32_t slot = 0) const = 0;
};

class Texture2D : public Texture {
 public:
  static memory::Shared<Texture2D> Create(const std::string& path);
};

}  // namespace renderer
}  // namespace core
}  // namespace engine

#endif  // ENGINE_SRC_CORE_RENDERER_TEXTURE_H_
