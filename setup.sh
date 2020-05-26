ROOT_DIR="$(git rev-parse --show-toplevel)"
pushd $ROOT_DIR

# Install pre-commit configs for working on the project.
pre-commit install

# Load git submodules.
git submodule update --init

popd  # ROOT_DIR
