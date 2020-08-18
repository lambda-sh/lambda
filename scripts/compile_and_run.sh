# compile_and_run.sh
#
# Compile and run one of the tools that the engine comes provided with in the
# tools folder.
ROOT_DIR="$(git rev-parse --show-toplevel)"
pushd $ROOT_DIR

# This sources the lambda utility script.
source scripts/lambda.sh

# -------------------------

export CC=gcc CXX=g++

if [ "$1" = "Release" ] || [ "$1" = "Debug" ]; then
    LAMBDA_INFO "Compiling a $1 build for the engine."
    cmake \
        -DCMAKE_BUILD_TYPE="$1" \
        -DDISTRIBUTION_BUILD=False \
        -DENGINE_DEVELOPMENT_MODE=True .
elif [ "$1" = "Dist" ]; then
    echo "Compiling a Distribution build."
    LAMBDA_INFO "Compiling a distribution build for the engine."
    cmake -DCMAKE_BUILD_TYPE="Release" -DDISTRIBUTION_BUILD=True .
else
    LAMBDA_FATAL "You need to pass a build type in order to compile a tool."
fi

LAMBDA_INFO "Compiling the engine with make -j 8."
make -j 8

# Go to the output binary and run it.
pushd "$1"/bin
./app
popd  # "$1/bin"
popd  # ROOT_DIR

LAMBDA_INFO "The tool and engine have been shutdown."
