# setup.sh
#
# Setup lambda for development. This currently doesn't do too much, but should
# be run at least ONCE after cloning the repository.
ROOT_DIR="$(git rev-parse --show-toplevel)"
pushd $ROOT_DIR

# Install pre-commit configs for working on the project.
pre-commit install

# Load git submodules.
git submodule update --init

popd  # ROOT_DIR
