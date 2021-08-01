#include "Lambda/platform/opengl/OpenGLShader.h"

#include <array>
#include <string>
#include <vector>
#include <fstream>

#include <Lambda/platform/glad/Glad.h>
#include <glm/gtc/type_ptr.hpp>

#include "Lambda/lib/Assert.h"
#include "Lambda/lib/Log.h"

namespace lambda::platform::opengl {

namespace {

/// @brief Internal OpenGL tool used for obtaining the shader type directly
/// from a string
///
/// Is not used externally.
static GLenum ShaderTypeFromString(const std::string& shader_type) {
  if (shader_type == "vertex") {
    return GL_VERTEX_SHADER;
  } else if (shader_type == "fragment" || shader_type == "pixel") {
    return GL_FRAGMENT_SHADER;
  } else {
    return GL_INVALID_ENUM;
  }
}

}  // namespace

OpenGLShader::OpenGLShader(const std::string& path) {
  std::string shader_source = ReadFile(path);
  const std::unordered_map<GLenum, std::string> shader_source_map =
      PreProcess(shader_source);
  Compile(shader_source_map);

  auto last_slash = path.find_last_of("/\\");

  // Remove the last slash if necessary.
  if (last_slash == std::string::npos) {
    last_slash = 0;
  } else {
    last_slash += 1;
  }

  const auto last_extension = path.rfind('.');
  unsigned long long shader_name_length;

  // Trim the extension if necessary.
  if (last_extension == std::string::npos) {
    shader_name_length = path.size() - last_slash;
  } else {
    shader_name_length = last_extension - last_slash;
  }

  name_ = path.substr(last_slash, shader_name_length);
}

OpenGLShader::OpenGLShader(
    const std::string& name,
    const std::string& vertex_source,
    const std::string& fragment_source) {
  std::unordered_map<GLenum, std::string> shader_source_map;

  shader_source_map[GL_VERTEX_SHADER] = vertex_source;
  shader_source_map[GL_FRAGMENT_SHADER] = fragment_source;

  Compile(shader_source_map);
  name_ = name;
}

OpenGLShader::~OpenGLShader() {
  glDeleteProgram(renderer_ID_);
}

std::string OpenGLShader::ReadFile(const std::string& path) {
  std::ifstream shader_file(path, std::ios::in | std::ios::binary);
  std::string result;

  if (shader_file) {
    shader_file.seekg(0, std::ios::end);
    result.resize(shader_file.tellg());
    shader_file.seekg(0, std::ios::beg);
    shader_file.read(&result[0], result.size());
  } else {
   LAMBDA_CORE_ERROR("Could not open the file: '{0}'", path);
  }

  return result;
}

std::unordered_map<GLenum, std::string> OpenGLShader::PreProcess(
    const std::string& shader_source) {
  std::unordered_map<GLenum, std::string> shader_source_map;

  const char* type_token = "#type";
  const size_t type_token_length = strlen(type_token);
  size_t position = shader_source.find(type_token, 0);

  while (position != std::string::npos) {
    // Find the end of type declaration.
    const size_t end_of_line = shader_source.find_first_of("\r\n", position);
    LAMBDA_CORE_ASSERT(end_of_line != std::string::npos, "Syntax error", "");

    // Read the shader type in and assert that it's a valid type.
    size_t shader_declaration_start = position + type_token_length + 1;
    std::string shader_type_str = shader_source.substr(
        shader_declaration_start, end_of_line - shader_declaration_start);

    GLenum shader_type_enum = ShaderTypeFromString(shader_type_str);
    LAMBDA_CORE_ASSERT(
        shader_type_enum != GL_INVALID_ENUM,
        "Invalid shader type specified: {0}",
        shader_type_str);

    // Find the start of the next line and then find the next type
    // declaration (If available)
    size_t next_line_start = shader_source.find_first_not_of(
        "\r\n", end_of_line);
    position = shader_source.find(type_token, next_line_start);

    // Determine the size of the shader currently being read.
    size_t shader_content_size =
        position - (
            next_line_start == std::string::npos ?
                shader_source.size() - 1 : next_line_start);

    shader_source_map[shader_type_enum] =
        shader_source.substr(next_line_start, shader_content_size);
  }

  return shader_source_map;
}

void OpenGLShader::Compile(
    const std::unordered_map<GLenum, std::string>& shader_source_map) {
  LAMBDA_CORE_ASSERT(shader_source_map.size() <= 3, "Too many shaders loaded")
  GLuint program = glCreateProgram();
  std::array<GLuint, 3> gl_shader_ids{};

  int shader_count = 0;
  for (auto& pair : shader_source_map) {
    GLenum shader_type = pair.first;
    const std::string& shader_source = pair.second;

    GLuint shader_ID = glCreateShader(shader_type);
    const GLchar* shader_program = shader_source.c_str();

    int has_compiled = GL_FALSE;
    glShaderSource(shader_ID, 1, &shader_program, 0);
    glCompileShader(shader_ID);
    glGetShaderiv(shader_ID, GL_COMPILE_STATUS, &has_compiled);

    if (has_compiled == GL_FALSE) {
      int maxLength = 0;
      glGetShaderiv(shader_ID, GL_INFO_LOG_LENGTH, &maxLength);
      std::vector<char> info_log(maxLength);
      glGetShaderInfoLog(shader_ID, maxLength, &maxLength, &info_log[0]);

      glDeleteShader(shader_ID);
      LAMBDA_CORE_ERROR(
          "Shader compilation failure for type{0}: {1}",
          shader_type,
          info_log.data());
    }

    glAttachShader(program, shader_ID);
    gl_shader_ids[shader_count] = shader_ID;
    ++shader_count;
  }

  glLinkProgram(program);
  auto program_linked = GL_FALSE;
  glGetProgramiv(program, GL_LINK_STATUS, &program_linked);

  if (program_linked == GL_FALSE) {
    int maxLength = 0;
    glGetProgramiv(program, GL_INFO_LOG_LENGTH, &maxLength);

    std::vector<char> info_log(maxLength);
    glGetProgramInfoLog(program, maxLength, &maxLength, &info_log[0]);

    glDeleteProgram(program);

    for (auto id : gl_shader_ids) {
      glDeleteShader(id);
    }

    LAMBDA_CORE_ERROR("Linking failure: {0}", info_log.data());
  }

  for (auto id : gl_shader_ids) {
    glDetachShader(renderer_ID_, id);
  }

  renderer_ID_ = program;
}

void OpenGLShader::Bind() const {
  glUseProgram(renderer_ID_);
}

void OpenGLShader::Unbind() const {
  glUseProgram(0);
}

void OpenGLShader::SetBool(const std::string& name, const bool& value) {
  UploadUniformBool(name, value);
}

void OpenGLShader::SetFloat(const std::string& name, const float& value) {
  UploadUniformFloat(name, value);
}

void OpenGLShader::SetFloat2(const std::string& name, const glm::vec2& values) {
  UploadUniformFloat2(name, values);
}

void OpenGLShader::SetFloat3(const std::string& name, const glm::vec3& values) {
  UploadUniformFloat3(name, values);
}

void OpenGLShader::SetFloat4(const std::string& name, const glm::vec4& values) {
  UploadUniformFloat4(name, values);
}

void OpenGLShader::SetInt(const std::string& name, const int& value) {
  UploadUniformInt(name, value);
}

void OpenGLShader::SetInt2(const std::string& name, const glm::vec2& values) {
  UploadUniformInt2(name, values);
}

void OpenGLShader::SetInt3(const std::string& name, const glm::vec3& values) {
  UploadUniformInt3(name, values);
}

void OpenGLShader::SetInt4(const std::string& name, const glm::vec4& values) {
  UploadUniformInt4(name, values);
}

void OpenGLShader::SetMat3(const std::string& name, const glm::mat3& matrix) {
  UploadUniformMat4(name, matrix);
}

void OpenGLShader::SetMat4(const std::string& name, const glm::mat4& matrix) {
  UploadUniformMat4(name, matrix);
}

// ----------------------------- OPENGL SPECIFIC -------------------------------

void OpenGLShader::UploadUniformBool(
    const std::string& name, const bool& value) {
  GLint location = glGetUniformLocation(renderer_ID_, name.c_str());
  glUniform1i(location, value);
}

void OpenGLShader::UploadUniformFloat(
    const std::string& name, const float& value) {
  GLint location = glGetUniformLocation(renderer_ID_, name.c_str());
  glUniform1f(location, value);
}

void OpenGLShader::UploadUniformFloat2(
    const std::string& name, const glm::vec2& values) {
  const GLint location = glGetUniformLocation(renderer_ID_, name.c_str());
  glUniform2f(location, values.x, values.y);
}

void OpenGLShader::UploadUniformFloat3(
    const std::string& name, const glm::vec3& values) {
  const auto location = glGetUniformLocation(renderer_ID_, name.c_str());
  glUniform3f(location, values.x, values.y, values.z);
}

void OpenGLShader::UploadUniformFloat4(
    const std::string& name, const glm::vec4& values) {
  const auto location = glGetUniformLocation(renderer_ID_, name.c_str());
  glUniform4f(location, values.x, values.y, values.z, values.a);
}

void OpenGLShader::UploadUniformInt(
    const std::string& name, const int& value) {
  const auto location = glGetUniformLocation(renderer_ID_, name.c_str());
  glUniform1i(location, value);
}

void OpenGLShader::UploadUniformInt2(
    const std::string& name, const glm::vec2& values) {
  const auto location  = glGetUniformLocation(renderer_ID_, name.c_str());
  glUniform2i(location, values.x, values.y);
}

void OpenGLShader::UploadUniformInt3(
    const std::string& name, const glm::vec3& values) {
  const auto location = glGetUniformLocation(renderer_ID_, name.c_str());
  glUniform3i(location, values.x, values.y, values.z);
}

void OpenGLShader::UploadUniformInt4(
    const std::string& name, const glm::vec4& values) {
  const auto location = glGetUniformLocation(renderer_ID_, name.c_str());
  glUniform4i(location, values.x, values.y, values.z, values.a);
}

void OpenGLShader::UploadUniformMat3(
    const std::string& name, const glm::mat3& matrix) {
  const auto location = glGetUniformLocation(renderer_ID_, name.c_str());
  glUniformMatrix3fv(location, 1, GL_FALSE, glm::value_ptr(matrix));
}

void OpenGLShader::UploadUniformMat4(
    const std::string& name, const glm::mat4& matrix) {
  const auto location = glGetUniformLocation(renderer_ID_, name.c_str());
  glUniformMatrix4fv(location, 1, GL_FALSE, glm::value_ptr(matrix));
}

}  // namespace lambda::platform::opengl
