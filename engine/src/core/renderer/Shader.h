/**
 * @file engine/src/core/renderer/Shader.h
 * @brief Shader API to be used with the renderer.
 */
#ifndef ENGINE_SRC_CORE_RENDERER_SHADER_H_
#define ENGINE_SRC_CORE_RENDERER_SHADER_H_

#include <string>

#include <glm/glm.hpp>

namespace engine {
namespace renderer {

/**
 * @class Shader
 * @brief The abstract Shader API.
 */
class Shader {
 public:
  virtual ~Shader() = default;

  /**
   * @fn Bind
   * @brief Binds the shader to the current graphics context.
   */
  virtual void Bind() const = 0;

  /**
   * @fn Unbind
   * @brief Unbinds the shader from the current graphics context.
   */
  virtual void Unbind() const = 0;

  static Shader* Create(
      const std::string& vertex_source, const std::string& fragment_source);
 private:
  std::uint32_t renderer_ID_;
};

}  // namespace renderer
}  // namespace engine

#endif  // ENGINE_SRC_CORE_RENDERER_SHADER_H_
