# compile_lambda.sh
#
# Compiles the lambda library
ROOT_DIR="$(git rev-parse --show-toplevel)"
pushd "$ROOT_DIR" > /dev/null

# ----------------------------- LAMBDA-SH & ARGS ------------------------------

source lambda-sh/lambda.sh

LAMBDA_PARSE_ARG build Release "The type of build to produce."
LAMBDA_PARSE_ARG cores 8 "The amount of cores to use for compiling."
LAMBDA_PARSE_ARG c-compiler gcc "The compiler to use for C code."
LAMBDA_PARSE_ARG cpp-compiler g++ "The compiler to use for C++ code."
LAMBDA_PARSE_ARG os Linux "The operating system that lambda is being built for."

LAMBDA_COMPILE_ARGS $@

export CC="$LAMBDA_c_compiler" CXX="$LAMBDA_cpp_compiler"

LAMBDA_INFO "Attempting to Compile a $LAMBDA_build for lambda."

# ----------------------------------- CMAKE ------------------------------------

mkdir -p build
pushd build > /dev/null

if [ "$LAMBDA_build" = "Release" ] || [ "$LAMBDA_build" = "Debug" ]; then
    LAMBDA_INFO "Compiling a $LAMBDA_build build for the engine."
    cmake .. \
        -DCMAKE_BUILD_TYPE="$LAMBDA_build" \
        -DDISTRIBUTION_BUILD=False
elif [ "$LAMBDA_build" = "Dist" ]; then
    LAMBDA_INFO "Compiling a distribution build for the engine."
    cmake .. \
        -DCMAKE_BUILD_TYPE="Release" \
        -DDISTRIBUTION_BUILD=True
else
    LAMBDA_FATAL "You need to pass a build type in order to compile a tool."
fi


LAMBDA_ASSERT_LAST_COMMAND_OK \
    "Couldn't generate the cmake files necessary for compiling lambda."

# ----------------------------------- BUILD ------------------------------------

if [ "$LAMBDA_os" = "Linux" ] || [ "$LAMBDA_os" = "Macos" ]; then
    make -j "$LAMBDA_cores"
elif [ "$LAMBDA_os" = "Windows" ]; then
    MSBuild.exe "lambda.sln" //t:Rebuild //p:Configuration=$LAMBDA_build
fi

LAMBDA_ASSERT_LAST_COMMAND_OK "Couldn't successfully compile lambda."
LAMBDA_INFO "Successfully compiled lambda"

popd > /dev/null  # build
popd > /dev/null  # ROOT_DIR
