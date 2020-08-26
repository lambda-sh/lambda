/// @file engine/src/Engine.h
/// @brief The entrypoint into the game engine source code.
///
/// This exposes all engine headers for use of any application.
#ifndef LAMBDA_SRC_LAMBDA_LAMBDA_H_
#define LAMBDA_SRC_LAMBDA_LAMBDA_H_

// ------------------------------------ CORE -----------------------------------

#include "core/Application.h"
#include "core/OrthographicCameraController.h"
#include "core/events/Event.h"
#include "core/imgui/ImGuiLayer.h"
#include "core/layers/Layer.h"

// ------------------------------------ INPUT ----------------------------------

#include "core/input/Input.h"
#include "core/input/KeyCodes.h"
#include "core/input/MouseButtonCodes.h"

// ------------------------------------ UTIL -----------------------------------

#include "core/util/Assert.h"
#include "core/util/Log.h"
#include "core/util/Reverse.h"
#include "core/util/Time.h"

// ------------------------------------- IO ------------------------------------

#include "core/io/EventLoop.h"
#include "core/io/AsyncTask.h"

// ---------------------------------- MEMORY -----------------------------------

#include "core/memory/Pointers.h"

// --------------------------------- RENDERING ---------------------------------

#include "core/renderer/Buffer.h"
#include "core/renderer/OrthographicCamera.h"
#include "core/renderer/RenderCommand.h"
#include "core/renderer/Renderer.h"
#include "core/renderer/Shader.h"
#include "core/renderer/Texture.h"
#include "core/renderer/VertexArray.h"

#endif  // LAMBDA_SRC_LAMBDA_LAMBDA_H_
