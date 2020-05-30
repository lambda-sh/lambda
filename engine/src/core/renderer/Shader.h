/**
 * @file engine/src/core/renderer/Shader.h
 * @brief Shader API to be used with the renderer.
 */
#ifndef ENGINE_SRC_CORE_RENDERER_SHADER_H_
#define ENGINE_SRC_CORE_RENDERER_SHADER_H_

#include <string>

namespace engine {
namespace renderer {

/**
 * @class Shader
 * @brief The Shader API.
 *
 * Expects sources to be strings that are designed for the shading language
 * that is being used in the API.
 */
class Shader {
 public:
  /**
   * @fn Shader
   * @brief Instantiates a Shader with vertex and fragment source strings.
   *
   * Source strings are expected to be written for the shading language that
   * the current graphics API is using to compile shaders.
   */
  Shader(const std::string& vertexSource, const std::string& fragmentSource);
  ~Shader();

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

 private:
  std::uint32_t renderer_ID_;
};

}  // namespace renderer
}  // namespace engine

#endif  // ENGINE_SRC_CORE_RENDERER_SHADER_H_
