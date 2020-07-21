#include "platform/opengl/OpenGLTexture.h"

#include <glad/glad.h>
#include <stb_image.h>

#include "core/util/Assert.h"

namespace engine {
namespace platform {
namespace opengl {

OpenGLTexture2D::OpenGLTexture2D(const std::string& path) : path_(path) {
  int width, height, channels;
  stbi_uc* data = stbi_load(path_.c_str(), &width, &height, &channels, 0);
  ENGINE_CORE_ASSERT(data, "Failed to load image");
  width_ = width;
  height_ = height;

  // Create the texture and specify some meta information about it.
  glCreateTextures(GL_TEXTURE_2D, 1, &renderer_ID_);
  glTextureStorage2D(renderer_ID_, 1, GL_RGB8, width_, height_);

  // Set the upscaling and downscaling functions to be linear.
  glTextureParameteri(renderer_ID_, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
  glTextureParameteri(renderer_ID_, GL_TEXTURE_MAG_FILTER, GL_LINEAR);
  glTextureSubImage2D(
      renderer_ID_, 0, 0, 0, width_, height_, GL_RGB, GL_UNSIGNED_BYTE, data);

  // Free the memory of the image now that it has been loaded into the GPU.
  stbi_image_free(data);
}

OpenGLTexture2D::~OpenGLTexture2D() {}

// Default slot is always 0.
void OpenGLTexture2D::Bind(uint32_t slot) const {
  glBindTextureUnit(slot, renderer_ID_);
}

}  // namespace opengl
}  // namespace platform
}  // namespace engine
