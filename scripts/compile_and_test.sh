# run_all_tests.sh
#
# Compile and run one of the tools that the engine comes provided with in the
# tools folder. This has been built and tested with bash 5.0.18
ROOT_DIR="$(git rev-parse --show-toplevel)"
pushd $ROOT_DIR > /dev/null

# -------------------- SETUP ARGUMENTS AND LAMBDA SHELL ------------------------

source lambda-sh/lambda.sh

LAMBDA_PARSE_ARG build Release "The type of build to produce."
LAMBDA_PARSE_ARG cores 8 "The amount of cores to use for compiling."
LAMBDA_PARSE_ARG c-compiler gcc "The compiler to use for C code."
LAMBDA_PARSE_ARG cpp-compiler g++ "The compiler to use for C++ code."
LAMBDA_PARSE_ARG os "Linux" "The operating system to build for."

LAMBDA_COMPILE_ARGS $@

# -------------------- COMPILE THE ENGINE AND ALL TOOLS ------------------------

export CC=$LAMBDA_c_compiler CXX=$LAMBDA_cpp_compiler

mkdir -p build
pushd build > /dev/null

if [ "$LAMBDA_build" = "Release" ] || [ "$LAMBDA_build" = "Debug" ]; then
    LAMBDA_INFO "Compiling a $LAMBDA_build build for the engine."
    cmake .. \
        -DCMAKE_BUILD_TYPE="$LAMBDA_build" \
        -DLAMBDA_ENGINE_BUILD_TESTS=ON \
        -G Ninja
else
    LAMBDA_FATAL "You need to pass a build type in order to compile a tool."
fi

LAMBDA_ASSERT_LAST_COMMAND_OK \
    "Couldn't generate the build files necessary for compiling lambda."

# ----------------------------------- BUILD ------------------------------------

if [ "$LAMBDA_os" = "Linux" ] || [ "$LAMBDA_os" = "Macos" ]; then
    ninja
    LAMBDA_ASSERT_LAST_COMMAND_OK "Failed to compile Lambda."

    # If using wsl2 & wslg, export latest opengl versions for mesa.
    if grep -q "WSL2" <<< "$(uname -srm)"; then
        export MESA_GL_VERSION_OVERRIDE=4.5
        export MESA_GLSL_VERSION_OVERRIDE=450
    fi

elif [ "$LAMBDA_os" = "Windows" ]; then
    MSBuild.exe "lambda.sln" //t:Rebuild //p:Configuration=$LAMBDA_build
    LAMBDA_ASSERT_LAST_COMMAND_OK "Failed to compile Lambda."
fi

# ------------------------------------ RUN -------------------------------------

pushd lambda/tests > /dev/null
./lambda_tests
popd > /dev/null  # lambda/tests

popd > /dev/null  # build
popd > /dev/null  # ROOT_DIR

LAMBDA_INFO "$LAMBDA_tool and engine have been shutdown."
# run_all_tests.sh
