#include "Lambda/platform/opengl/OpenGLTexture.h"

#include <glad/glad.h>
#include <stb_image.h>

#include "Lambda/core/util/Assert.h"

namespace lambda {
namespace platform {
namespace opengl {

OpenGLTexture2D::OpenGLTexture2D(const std::string& path) : path_(path) {
  int width, height, channels;

  // Load the texture from the bottom up, as that's how OpenGL expects to render
  // textures.
  stbi_set_flip_vertically_on_load(1);
  stbi_uc* data = stbi_load(path_.c_str(), &width, &height, &channels, 0);
  LAMBDA_CORE_TRACE("Attempting to load: {0}", path_);
  LAMBDA_CORE_ASSERT(data, "Failed to load the image: {0}", path_);
  width_ = width;
  height_ = height;

  // Pixel size and type. Needed for allocating the correct amount of
  // memory with OpenGL.
  int size_format = 0, type_format = 0;
  switch(channels) {
    case 4:
      size_format = GL_RGBA8;
      type_format = GL_RGBA;
      break;
    case 3:
      size_format = GL_RGB8;
      type_format = GL_RGB;
  }

  LAMBDA_CORE_ASSERT(
      size_format && type_format, "Pixel format for {0} not supported.",
      path_);

  // Create the texture and specify some meta information about it.
  glCreateTextures(GL_TEXTURE_2D, 1, &renderer_ID_);
  glTextureStorage2D(renderer_ID_, 1, size_format, width_, height_);

  // Set the upscaling and downscaling functions to be linear.
  glTextureParameteri(renderer_ID_, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
  glTextureParameteri(renderer_ID_, GL_TEXTURE_MAG_FILTER, GL_NEAREST);
  glTextureSubImage2D(
      renderer_ID_,
      0,
      0,
      0,
      width_,
      height_,
      type_format,
      GL_UNSIGNED_BYTE,
      data);

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
}  // namespace lambda
