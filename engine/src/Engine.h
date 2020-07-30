/**
 * @file engine/src/Engine.h
 * @brief The entrypoint into the game engine source code.
 *
 * This exposes all engine headers for use of any application.
 */
#ifndef ENGINE_SRC_ENGINE_H_
#define ENGINE_SRC_ENGINE_H_

// ------------------------------------ CORE -----------------------------------

#include "core/Application.h"
#include "core/Input.h"
#include "core/KeyCodes.h"
#include "core/KeyCodes.h"
#include "core/layers/Layer.h"
#include "core/MouseButtonCodes.h"
#include "core/events/Event.h"
#include "core/imgui/ImGuiLayer.h"

// ------------------------------------ UTIL -----------------------------------

#include "core/util/Assert.h"
#include "core/util/Log.h"
#include "core/util/Reverse.h"
#include "core/util/Time.h"


// ------------------------------------- IO ------------------------------------

#include "core/io/EventLoop.h"

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

// --------------------------------- ENTRYPOINT --------------------------------

#include "core/Entrypoint.h"

#endif  // ENGINE_SRC_ENGINE_H_
