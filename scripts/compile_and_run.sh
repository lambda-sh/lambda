# compile_and_run.sh
#
# Compile and run one of the tools that the engine comes provided with in the
# tools folder. This has been built and tested with bash 5.0.18
ROOT_DIR="$(git rev-parse --show-toplevel)"
pushd $ROOT_DIR

# -------------------- SETUP ARGUMENTS AND LAMBDA SHELL ------------------------

source scripts/lambda.sh

LAMBDA_PARSE_ARG tool sandbox "The tool to compile and run."
LAMBDA_PARSE_ARG build Release "The type of build to produce."
LAMBDA_PARSE_ARG cores 8 "The amount of cores to use for compiling."
LAMBDA_PARSE_ARG c-compiler gcc "The compiler to use for C code."
LAMBDA_PARSE_ARG cpp-compiler g++ "The compiler to use for C++ code."

LAMBDA_COMPILE_ARGS $@

# -------------------- COMPILE THE ENGINE AND ALL TOOLS ------------------------

export CC=$LAMBDA_c_compiler CXX=$LAMBDA_cpp_compiler

if [ $LAMBDA_build = "Release" ] || [ $LAMBDA_build = "Debug" ]; then
    LAMBDA_INFO "Compiling a $LAMBDA_build build for the engine."
    cmake \
        -DCMAKE_BUILD_TYPE="$LAMBDA_build" \
        -DDISTRIBUTION_BUILD=False \
        -DWITH_TOOL="$LAMBDA_tool" \
        -DENGINE_DEVELOPMENT_MODE=True .
elif [ "$LAMBDA_build" = "Dist" ]; then
    LAMBDA_INFO "Compiling a distribution build for the engine."
    cmake \
        -DCMAKE_BUILD_TYPE="Release" \
        -DWITH_TOOL="$LAMBDA_tool" \
        -DDISTRIBUTION_BUILD=True .
else
    LAMBDA_FATAL "You need to pass a build type in order to compile a tool."
fi

LAMBDA_INFO "Compiling the engine with make -j 8."
make -j $LAMBDA_cores

# ------------------------- RUN THE REQUESTED TOOL ----------------------------


# Go to the output binary and run it.
pushd "builds/$LAMBDA_build/bin"
./"$LAMBDA_tool"
popd  # "builds/$LAMDA_build/bin"

popd  # ROOT_DIR

LAMBDA_INFO "$LAMBDA_tool and engine have been shutdown."
