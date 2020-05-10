#ifndef ENGINE_SRC_CORE_RENDERER_SHADER_H_
#define ENGINE_SRC_CORE_RENDERER_SHADER_H_

#include <string>

namespace engine {
namespace renderer {

class Shader {
 public:
  Shader(const std::string& vertexSource, const std::string& fragmentSource);
  ~Shader();

  void Bind() const;
  void Unbind() const;
 private:
  std::uint32_t renderer_ID_;
};

}  // namespace renderer
}  // namespace engine

#endif  // ENGINE_SRC_CORE_RENDERER_SHADER_H_
