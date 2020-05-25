ROOT_DIR="$(git rev-parse --show-toplevel)"
pushd $ROOT_DIR

export CC=gcc CXX=g++

if [ "$1" = "Release" ] || [ "$1" = "Debug" ]; then
    echo "Compiling a $1 build."
    cmake \
        -DCMAKE_BUILD_TYPE="$1" \
        -DDISTRIBUTION_BUILD=False \
        -DENGINE_DEVELOPMENT_MODE=True .
elif [ "$1" = "Dist" ]; then
    echo "Compiling a Distribution build."
    cmake -DCMAKE_BUILD_TYPE="Release" -DDISTRIBUTION_BUILD=True .
else
    echo "Incorrect build type passed."
    popd
    exit
fi

make -j 8
./"$1"/bin/app

popd  # ROOT_DIR
