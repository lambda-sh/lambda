# compile_and_run.sh
#
# Compile and run one of the tools that the engine comes provided with in the
# tools folder. This has been built and tested with bash 5.0.18
ROOT_DIR="$(git rev-parse --show-toplevel)"
pushd $ROOT_DIR > /dev/null

# -------------------- SETUP ARGUMENTS AND LAMBDA SHELL ------------------------

source lambda-sh/lambda.sh

lambda_args_add \
    --name build \
    --default Release \
    --description "The type of build to produce."

lambda_args_add \
    --name cores \
    --default 8 \
    --description "The amount of cores to use for compilation."

lambda_args_add \
    --name c-compiler \
    --default gcc \
    --description "The compiler to use for C."

lambda_args_add \
    --name cpp-compiler \
    --default g++ \
    --description "The compiler to use for C++."

lambda_args_add \
    --name os \
    --default "Linux" \
    --description \
        "The operating system being built for. (MacOS, Windows, Linux)"

lambda_args_add \
    --name tool \
    --default "sandbox" \
    --description "The tool to run. (Named after tools in the tool folder)"


lambda_args_compile "$@"

# -------------------- COMPILE THE ENGINE AND ALL TOOLS ------------------------

export CC=$LAMBDA_c_compiler CXX=$LAMBDA_cpp_compiler

pushd archive/lambda_cpp > /dev/null
mkdir -p build
pushd build > /dev/null

if [ "$LAMBDA_build" = "Release" ] || [ "$LAMBDA_build" = "Debug" ]; then
    lambda_log_info "Compiling a $LAMBDA_build build for the engine."
    cmake .. \
        -DCMAKE_BUILD_TYPE="$LAMBDA_build" \
        -DDISTRIBUTION_BUILD=False \
        -DLAMBDA_TOOLS_BUILD_SANDBOX=ON \
        -DLAMBDA_TOOLS_BUILD_MATHBOX=ON \
        -DLAMBDA_TOOLS_BUILD_DEMO=ON \
        -G Ninja
elif [ "$LAMBDA_build" = "Dist" ]; then
    lambda_log_info "Compiling a distribution build for the engine."
    cmake .. \
        -DCMAKE_BUILD_TYPE="Release" \
        -DDISTRIBUTION_BUILD=True \
        -DLAMBDA_TOOLS_BUILD_SANDBOX=ON \
        -DLAMBDA_TOOLS_BUILD_MATHBOX=ON \
        -DLAMBDA_TOOLS_BUILD_DEMO=ON \
        -G Ninja
else
    lambda_log_fatal "You need to pass a build type in order to compile a tool."
fi

lambda_assert_last_command_ok \
    "Couldn't generate the cmake files necessary for compiling lambda."

# ----------------------------------- BUILD ------------------------------------

if [ "$LAMBDA_os" = "Linux" ] || [ "$LAMBDA_os" = "Macos" ]; then
    ninja
    lambda_assert_last_command_ok "Failed to compile Lambda."

    # If using wsl2 & wslg, export latest opengl versions for mesa.
    if grep -q "WSL2" <<< "$(uname -srm)"; then
        export MESA_GL_VERSION_OVERRIDE=4.5
        export MESA_GLSL_VERSION_OVERRIDE=450
    fi

elif [ "$LAMBDA_os" = "Windows" ]; then
    MSBuild.exe "lambda.sln" //t:Rebuild //p:Configuration=$LAMBDA_build
    lambda_assert_last_command_ok "Failed to compile Lambda."
fi

# ------------------------------------ RUN -------------------------------------

pushd "tools/$LAMBDA_tool" > /dev/null
lambda_assert_last_command_ok "Couldn't access the tools directory."

./"$LAMBDA_tool"
popd > /dev/null

popd > /dev/null  # "build"
popd > /dev/null  # "archive/lambda_cpp"
popd > /dev/null  # ROOT_DIR

lambda_log_info "$LAMBDA_tool and engine have been shutdown."
