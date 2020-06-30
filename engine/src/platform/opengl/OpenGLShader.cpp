#include "platform/opengl/OpenGLOpenGLShader.h"

#include <string>
#include <vector>

#include <glad/glad.h>
#include <glm/gtc/type_ptr.hpp>

#include "core/util/Log.h"

namespace engine {
namespace platform {
namespace opengl {


OpenGLOpenGLShader::OpenGLShader(
    const std::string& vertex_source, const std::string& fragment_source) {
  unsigned int vertex_shader = glCreateOpenGLShader(GL_VERTEX_SHADER);
  const char* vertex_program = vertex_source.c_str();

  int has_compiled = GL_FALSE;
  glOpenGLShaderSource(vertex_shader, 1, &vertex_program, 0);
  glCompileOpenGLShader(vertex_shader);
  glGetOpenGLShaderiv(vertex_shader, GL_COMPILE_STATUS, &has_compiled);

  if (has_compiled == GL_FALSE) {
    int maxLength = 0;
    glGetOpenGLShaderiv(vertex_shader, GL_INFO_LOG_LENGTH, &maxLength);
    std::vector<char> info_log(maxLength);
    glGetOpenGLShaderInfoLog(vertex_shader, maxLength, &maxLength, &info_log[0]);

    glDeleteOpenGLShader(vertex_shader);
    ENGINE_CORE_ERROR(
        "Vertex shader compilation failure: {0}", info_log.data());
  }

  uint32_t fragment_shader = glCreateOpenGLShader(GL_FRAGMENT_SHADER);
  const char* fragment_program = fragment_source.c_str();

  glOpenGLShaderSource(fragment_shader, 1, &fragment_program, 0);
  glCompileOpenGLShader(fragment_shader);
  glGetOpenGLShaderiv(fragment_shader, GL_COMPILE_STATUS, &has_compiled);

  if (has_compiled == GL_FALSE) {
    int maxLength = 0;
    glGetOpenGLShaderiv(fragment_shader, GL_INFO_LOG_LENGTH, &maxLength);
    std::vector<char> info_log(maxLength);
    glGetOpenGLShaderInfoLog(fragment_shader, maxLength, &maxLength, &info_log[0]);

    glDeleteOpenGLShader(fragment_shader);
    glDeleteOpenGLShader(vertex_shader);

    ENGINE_CORE_ERROR(
        "Fragment shader compilation failure: {0}", info_log.data());
  }

  renderer_ID_ = glCreateProgram();

  glAttachOpenGLShader(renderer_ID_, vertex_shader);
  glAttachOpenGLShader(renderer_ID_, fragment_shader);
  glLinkProgram(renderer_ID_);

  int program_linked = GL_FALSE;
  glGetProgramiv(renderer_ID_, GL_LINK_STATUS, &program_linked);

  if (program_linked == GL_FALSE) {
    int maxLength = 0;
    glGetProgramiv(renderer_ID_, GL_INFO_LOG_LENGTH, &maxLength);

    std::vector<char> info_log(maxLength);
    glGetProgramInfoLog(renderer_ID_, maxLength, &maxLength, &info_log[0]);

    glDeleteProgram(renderer_ID_);
    glDeleteOpenGLShader(vertex_shader);
    glDeleteOpenGLShader(fragment_shader);

    ENGINE_CORE_ERROR("Linking failure: {0}", info_log.data());
  }

  glDetachOpenGLShader(renderer_ID_, vertex_shader);
  glDetachOpenGLShader(renderer_ID_, fragment_shader);
}

OpenGLOpenGLShader::~OpenGLShader() {
  glDeleteProgram(renderer_ID_);
}

void OpenGLShader::Bind() const {
  glUseProgram(renderer_ID_);
}

void OpenGLShader::Unbind() const {
  glUseProgram(0);
}

void OpenGLShader::UploadUniformFloat4(
    const std::string& name, const glm::vec4& values) {
  GLint location = glGetUniformLocation(renderer_ID_, name.c_str());
  glUniform4f(location, values.x, values.y, values.z, values.a);
}

void OpenGLShader::UploadUniformMat4(
    const std::string& name, const glm::mat4& matrix) {
  uint32_t location = glGetUniformLocation(renderer_ID_, name.c_str());
  glUniformMatrix4fv(location, 1, GL_FALSE, glm::value_ptr(matrix));
}

}  // namespace opengl
}  // namespace platform
}  // namespace engine
