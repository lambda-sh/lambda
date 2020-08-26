
/// @file Shader.h
/// @brief Shader API to be used with the renderer.

#ifndef LAMBDA_SRC_LAMBDA_CORE_RENDERER_SHADER_H_
#define LAMBDA_SRC_LAMBDA_CORE_RENDERER_SHADER_H_

#include <string>
#include <unordered_map>

#include <glm/glm.hpp>

#include "core/memory/Pointers.h"

namespace lambda {
namespace core {
namespace renderer {

/// @brief A generic Shader API.
class Shader {
 public:
  virtual ~Shader() = default;

  /// @brief Binds the shader to the GPU.
  virtual void Bind() const = 0;

  /// @brief Unbinds the shader from the GPU.
  virtual void Unbind() const = 0;

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
