#include "core/renderer/Shader.h"

#include <vector>

#include <glad/glad.h>

#include "core/Log.h"

namespace engine {
namespace renderer {

Shader::Shader(
    const std::string& vertex_source, const std::string& fragment_source) {

  // Create and compile the vertex shader
  unsigned int vertex_shader = glCreateShader(GL_VERTEX_SHADER);
  const char* vertex_program = vertex_source.c_str();
  glShaderSource(vertex_shader, 1, &vertex_program, 0);
  glCompileShader(vertex_shader);

  // Get the status of the compilation.
  int isCompiled = 0;
  glGetShaderiv(vertex_shader, GL_COMPILE_STATUS, &isCompiled);

  if (isCompiled == GL_FALSE) {
    int maxLength = 0;
    glGetShaderiv(vertex_shader, GL_INFO_LOG_LENGTH, &maxLength);
    std::vector<char> info_log(maxLength);
    glGetShaderInfoLog(vertex_shader, maxLength, &maxLength, &info_log[0]);

    glDeleteShader(vertex_shader);
    ENGINE_CORE_ERROR(
        "Vertex shader compilation failure: {0}", info_log.data());
  }

  // Create and compile the openGL shader.
  uint32_t fragment_shader = glCreateShader(GL_FRAGMENT_SHADER);
  const char* fragment_program = fragment_source.c_str();
  glShaderSource(fragment_shader, 1, &fragment_program, 0);
  glCompileShader(fragment_shader);

  glGetShaderiv(fragment_shader, GL_COMPILE_STATUS, &isCompiled);
  if (isCompiled == GL_FALSE) {
    int maxLength = 0;
    glGetShaderiv(fragment_shader, GL_INFO_LOG_LENGTH, &maxLength);
    std::vector<char> info_log(maxLength);
    glGetShaderInfoLog(fragment_shader, maxLength, &maxLength, &info_log[0]);

    glDeleteShader(fragment_shader);
    glDeleteShader(vertex_shader);

    ENGINE_CORE_ERROR(
        "Fragment shader compilation failure: {0}", info_log.data());
  }

  // Create and link our renderer_ID_ to the compiled shaders.
  renderer_ID_ = glCreateProgram();
  glAttachShader(renderer_ID_, vertex_shader);
  glAttachShader(renderer_ID_, fragment_shader);
  glLinkProgram(renderer_ID_);

  // Note the different functions here: glGetProgram* instead of glGetShader*.
  int isLinked = 0;
  glGetProgramiv(renderer_ID_, GL_LINK_STATUS, &isLinked);

  if (isLinked == GL_FALSE) {
    int maxLength = 0;
    glGetProgramiv(renderer_ID_, GL_INFO_LOG_LENGTH, &maxLength);
    std::vector<char> info_log(maxLength);
    glGetProgramInfoLog(renderer_ID_, maxLength, &maxLength, &info_log[0]);

    // Cleanup.
    glDeleteProgram(renderer_ID_);
    glDeleteShader(vertex_shader);
    glDeleteShader(fragment_shader);

    // Use the info_log as you see fit.
    ENGINE_CORE_ERROR("Linking failure: {0}", info_log.data());
  }

  // Always detach shaders after a successful link.
  glDetachShader(renderer_ID_, vertex_shader);
  glDetachShader(renderer_ID_, fragment_shader);
}

Shader::~Shader() {
  glDeleteProgram(renderer_ID_);
}

void Shader::Bind() const {
  glUseProgram(renderer_ID_);
}

void Shader::Unbind() const {
  glUseProgram(0);
}

}  // namespace renderer
}  // namespace engine
