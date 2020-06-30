#include "core/renderer/Shader.h"

#include <string>
#include <vector>

#include <glad/glad.h>
#include <glm/gtc/type_ptr.hpp>

#include "core/util/Log.h"

namespace engine {
namespace renderer {

Shader::Shader(
    const std::string& vertex_source, const std::string& fragment_source) {
  unsigned int vertex_shader = glCreateShader(GL_VERTEX_SHADER);
  const char* vertex_program = vertex_source.c_str();

  int has_compiled = GL_FALSE;
  glShaderSource(vertex_shader, 1, &vertex_program, 0);
  glCompileShader(vertex_shader);
  glGetShaderiv(vertex_shader, GL_COMPILE_STATUS, &has_compiled);

  if (has_compiled == GL_FALSE) {
    int maxLength = 0;
    glGetShaderiv(vertex_shader, GL_INFO_LOG_LENGTH, &maxLength);
    std::vector<char> info_log(maxLength);
    glGetShaderInfoLog(vertex_shader, maxLength, &maxLength, &info_log[0]);

    glDeleteShader(vertex_shader);
    ENGINE_CORE_ERROR(
        "Vertex shader compilation failure: {0}", info_log.data());
  }

  uint32_t fragment_shader = glCreateShader(GL_FRAGMENT_SHADER);
  const char* fragment_program = fragment_source.c_str();

  glShaderSource(fragment_shader, 1, &fragment_program, 0);
  glCompileShader(fragment_shader);
  glGetShaderiv(fragment_shader, GL_COMPILE_STATUS, &has_compiled);

  if (has_compiled == GL_FALSE) {
    int maxLength = 0;
    glGetShaderiv(fragment_shader, GL_INFO_LOG_LENGTH, &maxLength);
    std::vector<char> info_log(maxLength);
    glGetShaderInfoLog(fragment_shader, maxLength, &maxLength, &info_log[0]);

    glDeleteShader(fragment_shader);
    glDeleteShader(vertex_shader);

    ENGINE_CORE_ERROR(
        "Fragment shader compilation failure: {0}", info_log.data());
  }

  renderer_ID_ = glCreateProgram();

  glAttachShader(renderer_ID_, vertex_shader);
  glAttachShader(renderer_ID_, fragment_shader);
  glLinkProgram(renderer_ID_);

  int program_linked = GL_FALSE;
  glGetProgramiv(renderer_ID_, GL_LINK_STATUS, &program_linked);

  if (program_linked == GL_FALSE) {
    int maxLength = 0;
    glGetProgramiv(renderer_ID_, GL_INFO_LOG_LENGTH, &maxLength);

    std::vector<char> info_log(maxLength);
    glGetProgramInfoLog(renderer_ID_, maxLength, &maxLength, &info_log[0]);

    glDeleteProgram(renderer_ID_);
    glDeleteShader(vertex_shader);
    glDeleteShader(fragment_shader);

    ENGINE_CORE_ERROR("Linking failure: {0}", info_log.data());
  }

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

void Shader::UploadUniformFloat4(
    const std::string& name, const glm::vec4& values) {
  GLint location = glGetUniformLocation(renderer_ID_, name.c_str());
  glUniform4f(location, values.x, values.y, values.z, values.a);
}

void Shader::UploadUniformMat4(
    const std::string& name, const glm::mat4& matrix) {
  uint32_t location = glGetUniformLocation(renderer_ID_, name.c_str());
  glUniformMatrix4fv(location, 1, GL_FALSE, glm::value_ptr(matrix));
}

}  // namespace renderer
}  // namespace engine
