/// @file engine/src/Engine.h
/// @brief The entrypoint into the game engine source code.
///
/// This exposes all engine headers for use of any application.
#ifndef LAMBDA_SRC_LAMBDA_LAMBDA_H_
#define LAMBDA_SRC_LAMBDA_LAMBDA_H_

// ------------------------------------ CORE -----------------------------------

#include <Lambda/core/Application.h>
#include <Lambda/core/OrthographicCameraController.h>
#include <Lambda/core/events/Event.h>
#include <Lambda/core/imgui/ImGuiLayer.h>
#include <Lambda/core/layers/Layer.h>

// ------------------------------------ INPUT ----------------------------------

#include <Lambda/core/input/Input.h>
#include <Lambda/core/input/KeyCodes.h>
#include <Lambda/core/input/MouseButtonCodes.h>

// ------------------------------------ UTIL -----------------------------------

#include <Lambda/core/util/Assert.h>
#include <Lambda/core/util/Log.h>
#include <Lambda/core/util/Reverse.h>
#include <Lambda/core/util/Time.h>

// ------------------------------------- IO ------------------------------------

#include <Lambda/core/io/EventLoop.h>
#include <Lambda/core/io/AsyncTask.h>

// ---------------------------------- MEMORY -----------------------------------

#include <Lambda/core/memory/Pointers.h>

// --------------------------------- RENDERING ---------------------------------

#include <Lambda/core/renderer/Buffer.h>
#include <Lambda/core/renderer/OrthographicCamera.h>
#include <Lambda/core/renderer/RenderCommand.h>
#include <Lambda/core/renderer/Renderer.h>
#include <Lambda/core/renderer/Shader.h>
#include <Lambda/core/renderer/Texture.h>
#include <Lambda/core/renderer/VertexArray.h>

// ----------------------------------- MATH ------------------------------------

#include <Lambda/math/Precision.h>
#include <Lambda/math/Vector.h>
#include <Lambda/math/plot/Graph.h>
#include <Lambda/math/shapes/Point.h>

#endif  // LAMBDA_SRC_LAMBDA_LAMBDA_H_
