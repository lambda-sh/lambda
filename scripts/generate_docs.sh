# This script is primarily for internal use. Official documentation will only
# be generated engineers that are part of the core development team for this
# project.

ROOT_DIR="$(git rev-parse --show-toplevel)"
pushd $ROOT_DIR

doxygen Doxyfile

popd  # ROOT_DIR
