/// @file OpenGLTexture.h
/// @brief The OpenGL Texture API.
///
/// Currently only supports 2D textures.
#ifndef LAMBDA_PLATFORM_OPENGL_OPENGLTEXTURE_H_
#define LAMBDA_PLATFORM_OPENGL_OPENGLTEXTURE_H_

#include <Lambda/platform/glad/Glad.h>

#include <string>

#include "Lambda/core/renderer/Texture.h"

namespace lambda {
namespace platform {
namespace opengl {

/// @brief THe opengl 2D texture implementation.
class OpenGLTexture2D : public core::renderer::Texture2D {
 public:
  OpenGLTexture2D(uint32_t width, uint32_t height);
  explicit OpenGLTexture2D(const std::string& path);
  virtual ~OpenGLTexture2D();

  /// @brief Get the width of the texture.
  uint32_t GetWidth() const override { return width_; }

  /// @brief Get the height of the texture.
  uint32_t GetHeight() const override { return height_; }

  void SetData(void* data, uint32_t size) override;

  /// @brief Bind the texture to a texture slot. (Default is 0)
  void Bind(uint32_t slot = 0) const override;

  /// @brief Unbind the current texture.
  void Unbind() const override;

 private:
  std::string path_;
  uint32_t height_;
  uint32_t renderer_ID_;
  uint32_t width_;
  GLenum size_format_, type_format_;
};

}  // namespace opengl
}  // namespace platform
}  // namespace lambda

#endif  // LAMBDA_PLATFORM_OPENGL_OPENGLTEXTURE_H_
