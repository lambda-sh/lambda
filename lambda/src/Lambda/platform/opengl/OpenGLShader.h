/**
 * @file engine/src/core/renderer/Shader.h
 * @brief Shader API to be used with the renderer.
 */
#ifndef LAMBDA_SRC_LAMBDA_PLATFORM_OPENGL_OPENGLSHADER_H_
#define LAMBDA_SRC_LAMBDA_PLATFORM_OPENGL_OPENGLSHADER_H_

#include <string>
#include <unordered_map>

#include <glad/glad.h>
#include <glm/glm.hpp>

#include "Lambda/core/renderer/Shader.h"

namespace lambda {
namespace platform {
namespace opengl {

/**
 * @class Shader
 * @brief The Shader API.
 *
 * Expects sources to be strings that are designed for the shading language
 * that is being used in the API.
 */
class OpenGLShader final : public core::renderer::Shader {
 public:
  /**
   * @fn OpenGLShader
   * @brief Instantiates an OpenGL Shader with vertex and fragment source
   * strings.
   */
  OpenGLShader(const std::string& path);
  OpenGLShader(
      const std::string& name,
      const std::string& vertex_source,
      const std::string& fragment_source);

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
   * @fn UploadUniformInt
   * @brief Upload an integer into the shader.
   */
  void UploadUniformInt(const std::string& name, const int& value);

  /**
   * @fn UploadUniformFloat
   * @brief Upload a float into the shader.
   */
  void UploadUniformFloat(const std::string& name, const float& value);

  /**
   * @fn UploadUniformFloat2
   * @brief Upload a uniform of 2 floats into the shader.
   */
  void UploadUniformFloat2(const std::string& name, const glm::vec2& values);

  /**
   * @fn UploadUniformFloat3
   * @brief Upload a uniform of 3 floats into the shader.
   */
  void UploadUniformFloat3(const std::string& name, const glm::vec3& values);

  /**
   * @fn UploadUniformFloat4
   * @brief Allows the uploading of uniform float 4s into the shader.
   */
  void UploadUniformFloat4(const std::string& name, const glm::vec4& matrix);

  /**
   * @fn UploadUniformMat3
   * @brief Upload a uniform of Matrix 3s into the shader.
   */
  void UploadUniformMat3(const std::string& name, const glm::mat3& matrix);

  /**
   * @fn UploadUniformMat4
   * @brief Upload a uniform of Matrix 4s into the shader.
   */
  void UploadUniformMat4(const std::string& name, const glm::mat4& matrix);

 private:
  std::string ReadFile(const std::string& path);
  std::unordered_map<GLenum, std::string> PreProcess(const std::string& shader_source);
  void Compile(const std::unordered_map<GLenum, std::string>& shader_source_map);

};

}  // namespace opengl
}  // namespace platform
}  // namespace lambda

#endif  // LAMBDA_SRC_LAMBDA_PLATFORM_OPENGL_OPENGLSHADER_H_
