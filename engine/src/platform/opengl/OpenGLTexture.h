/**
 * @file OpenGLTexture.h
 * @brief The OpenGL Texture API.
 *
 * Currently only supports 2D textures.
 */
#ifndef ENGINE_PLATFORM_OPENGL_OPENGLTEXTURE_H_
#define ENGINE_PLATFORM_OPENGL_OPENGLTEXTURE_H_

#include <bits/stdint-uintn.h>
#include <string>

#include "core/renderer/Texture.h"

namespace engine {
namespace platform {
namespace opengl {

class OpenGLTexture2D : public renderer::Texture2D {
 public:
  OpenGLTexture2D(const std::string& path);
  virtual ~OpenGLTexture2D();

  inline uint32_t GetWidth() const override { return width_; }
  inline uint32_t GetHeight() const override { return height_; }

  void Bind(uint32_t slot = 0) const override;

 private:
  std::string path_;
  uint32_t height_;
  uint32_t renderer_ID_;
  uint32_t width_;
};

}  // namespace opengl
}  // namespace platform
}  // namespace engine

#endif  // ENGINE_PLATFORM_OPENGL_OPENGLTEXTURE_H_

/**
 * @class engine::platform::opengl::OpenGLTexture2D
 * @brief The opengl 2D texture implementation.
 *
 */

/**
 * @function engine::platform::opengl::OpenGLTexture2D::GetWidth
 * @brief Get the width of the texure.
 */

/**
 * @function engine::platform::opengl::OpenGLTexture2D::GetHeight
 * @brief Get the height of the texure.
 */

/**
 * @function engine::platform::opengl::OpenGLTexture2D::Bind
 * @brief Bind the texture to the engine.
 */
