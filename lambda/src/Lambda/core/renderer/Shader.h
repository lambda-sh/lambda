
/// @file Shader.h
/// @brief Shader API to be used with the renderer.

#ifndef LAMBDA_SRC_LAMBDA_CORE_RENDERER_SHADER_H_
#define LAMBDA_SRC_LAMBDA_CORE_RENDERER_SHADER_H_

#include <string>
#include <unordered_map>

#include <glm/glm.hpp>

#include "Lambda/core/memory/Pointers.h"

namespace lambda {
namespace core {
namespace renderer {

/// @brief A generic Shader API.
///
/// @todo This class shouldn't rely on glm for mathematics, as this ties the
/// abstract shader API to opengl specific vector mathematics.
class Shader {
 public:
  virtual ~Shader() = default;

  /// @brief Binds the shader to the GPU.
  virtual void Bind() const = 0;

  /// @brief Unbinds the shader from the GPU.
  virtual void Unbind() const = 0;

  /// @brief Set a boolean within the shader.
  virtual void SetBool(const std::string& name, const bool& value) = 0;

  /// @brief Sets a float within the shader.
  virtual void SetFloat(const std::string& name, const float& value) = 0;

  /// @brief Sets a vector of 2 floats within the shader.
  virtual void SetFloat2(const std::string& name, const glm::vec2& vector) = 0;

  /// @brief Sets a vector of 3 floats within the shader.
  virtual void SetFloat3(const std::string& name, const glm::vec3& vector) = 0;

  /// @brief Sets a vector of 3 floats within the shader.
  virtual void SetFloat4(const std::string& name, const glm::vec4& vector) = 0;

  /// @brief Sets an integer within the shader.
  virtual void SetInt(const std::string& name, const int& value) = 0;

  /// @brief Sets a vector of 2 Integers within the shader.
  virtual void SetInt2(const std::string& name, const glm::vec2& vector) = 0;

  /// @brief Sets a vector of 3 Integers within the shader.
  virtual void SetInt3(const std::string& name, const glm::vec3& vector) = 0;

  /// @brief Sets a vector of 4 Integers within the shader.
  virtual void SetInt4(const std::string& name, const glm::vec4& vector) = 0;

  /// @brief Sets a 3x3 matrix within the shader.
  virtual void SetMat3(const std::string& name, const glm::mat3& matrix) = 0;

  /// @brief Sets a 4x4 matrix within the shader.
  virtual void SetMat4(const std::string& name, const glm::mat4& matrix) = 0;

  /// @brief Get the name of the shader.
  const std::string& GetName() { return name_; }

  /// @brief Create a shader given the path to the shader.
  static memory::Shared<Shader> Create(const std::string& path);

  /// @brief Create a shader given a shader name, vertex source, and fragment
  /// source all as strings.
  static memory::Shared<Shader> Create(
      const std::string& name,
      const std::string& vertex_source,
      const std::string& fragment_source);

 protected:
  uint32_t renderer_ID_;
  std::string name_;
};

/// @brief A library for managing many different shaders.
class ShaderLibrary {
 public:
  /// @brief Add a shader that has already been created into the library.
  void Add(const memory::Shared<Shader>& shader);

  /// @brief Add a shader that has already been created into the library.
  void Add(const std::string& name, const memory::Shared<Shader>& shader);

  /// @brief Get a shader from the library given the name of the shader.
  memory::Shared<Shader> Get(const std::string& name);

  /// @brief Load a shader into lambda through the path of the shader. (uses the
  /// name of the shader for naming.)
  memory::Shared<Shader> Load(const std::string& path);

  /// @brief Load a shader into lambda with a path to the shader and name to be
  /// used within the shader library.
  memory::Shared<Shader> Load(const std::string& name, const std::string& path);
 private:
  std::unordered_map<std::string, memory::Shared<Shader>> shader_mapping_;
};

}  // namespace renderer
}  // namespace core
}  // namespace lambda

#endif  // LAMBDA_SRC_LAMBDA_CORE_RENDERER_SHADER_H_
