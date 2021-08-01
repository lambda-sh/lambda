# compile_lambda.sh
#
# Compiles the lambda library
ROOT_DIR="$(git rev-parse --show-toplevel)"
pushd "$ROOT_DIR" > /dev/null

# ----------------------------- LAMBDA-SH & ARGS ------------------------------

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

lambda_args_compile "$@"

export CC="$LAMBDA_c_compiler" CXX="$LAMBDA_cpp_compiler"

lambda_log_info "Attempting to Compile a $LAMBDA_build for lambda."

# ----------------------------------- CMAKE ------------------------------------

mkdir -p build
pushd build > /dev/null

if [ "$LAMBDA_build" = "Release" ] || [ "$LAMBDA_build" = "Debug" ]; then
    lambda_log_info "Compiling a $LAMBDA_build build for the engine."
    cmake .. \
        -DCMAKE_BUILD_TYPE="$LAMBDA_build" \
        -DDISTRIBUTION_BUILD=False
elif [ "$LAMBDA_build" = "Dist" ]; then
    lambda_log_info "Compiling a distribution build for the engine."
    cmake .. \
        -DCMAKE_BUILD_TYPE="Release" \
        -DDISTRIBUTION_BUILD=True
else
    lambda_log_fatal "You need to pass a build type in order to compile a tool."
fi


lambda_assert_last_command_ok \
    "Couldn't generate the cmake files necessary for compiling lambda."

# ----------------------------------- BUILD ------------------------------------

if [ "$LAMBDA_os" = "Linux" ] || [ "$LAMBDA_os" = "Macos" ]; then
    ninja -j "$LAMBDA_cores"
elif [ "$LAMBDA_os" = "Windows" ]; then
    MSBuild.exe "lambda.sln" //t:Rebuild //p:Configuration=$LAMBDA_build
fi

lambda_assert_last_command_ok "Couldn't successfully compile lambda."
lambda_log_info "Successfully compiled lambda"

popd > /dev/null  # build
popd > /dev/null  # ROOT_DIR
