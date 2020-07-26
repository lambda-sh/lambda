/**
 * @file Shader.h
 * @brief Shader API to be used with the renderer.
 */
#ifndef ENGINE_SRC_CORE_RENDERER_SHADER_H_
#define ENGINE_SRC_CORE_RENDERER_SHADER_H_

#include <string>
#include <unordered_map>

#include <glm/glm.hpp>

#include "core/memory/Pointers.h"

namespace engine {
namespace renderer {

class Shader {
 public:
  virtual ~Shader() = default;

  virtual void Bind() const = 0;
  virtual void Unbind() const = 0;

  inline const std::string& GetName() { return name_; }

  static memory::Shared<Shader> Create(const std::string& path);
  static memory::Shared<Shader> Create(
      const std::string& name,
      const std::string& vertex_source,
      const std::string& fragment_source);

 protected:
  uint32_t renderer_ID_;
  std::string name_;
};

class ShaderLibrary {
 public:
  void Add(const memory::Shared<Shader>& shader);
  void Add(const std::string& name, const memory::Shared<Shader>& shader);

  memory::Shared<Shader> Get(const std::string& name);
  memory::Shared<Shader> Load(const std::string& path);
  memory::Shared<Shader> Load(const std::string& name, const std::string& path);
 private:
  std::unordered_map<std::string, memory::Shared<Shader>> shader_mapping_;
};

}  // namespace renderer
}  // namespace engine

#endif  // ENGINE_SRC_CORE_RENDERER_SHADER_H_

/**
 * @class engine::renderer::Shader
 * @brief The abstract Shader API.
 */

/**
 * @fn engine::renderer::Bind
 * @brief Binds the shader to the current graphics context.
 */

/**
 * @fn engine::renderer::Unbind
 * @brief Unbinds the shader from the current graphics context.
 */
