/**
 * @file engine/src/core/renderer/Shader.h
 * @brief Shader API to be used with the renderer.
 */
#ifndef ENGINE_SRC_PLATFORM_OPENGL_OPENGLSHADER_H_
#define ENGINE_SRC_PLATFORM_OPENGL_OPENGLSHADER_H_

#include <string>

#include <glm/glm.hpp>

namespace engine {
namespace platform {
namespace opengl {


/**
 * @class Shader
 * @brief The Shader API.
 *
 * Expects sources to be strings that are designed for the shading language
 * that is being used in the API.
 */
class OpenGLShader {
 public:
  /**
   * @fn Shader
   * @brief Instantiates a Shader with vertex and fragment source strings.
   *
   * Source strings are expected to be written for the shading language that
   * the current graphics API is using to compile shaders.
   */
  OpenGLShader(
      const std::string& vertexSource, const std::string& fragmentSource);
  ~OpenGLShader();

  /**
   * @fn Bind
   * @brief Binds the shader to the current graphics context.
   */
  void Bind() const;

  /**
   * @fn Unbind
   * @brief Unbinds the shader from the current graphics context.
   */
  void Unbind() const;

  /**
   * @fn UploadUniformMat4
   * @brief Allows the uploading of uniform4s into the shader.
   */
  void UploadUniformMat4(const std::string& name, const glm::mat4& matrix);

  /**
   * @fn UploadUniformFloat4
   * @brief Allows the uploading of uniform float 4s into the shader.
   */
  void UploadUniformFloat4(const std::string& name, const glm::vec4& matrix);

 private:
  std::uint32_t renderer_ID_;
};

}  // namespace opengl
}  // namespace platform
}  // namespace engine

#endif  // ENGINE_SRC_PLATFORM_OPENGL_OPENGLSHADER_H_
