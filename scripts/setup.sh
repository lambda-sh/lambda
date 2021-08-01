# setup.sh
#
# Setup lambda for development. This currently doesn't do too much, but should
# be run at least ONCE after cloning the repository.
ROOT_DIR="$(git rev-parse --show-toplevel)"
pushd "$ROOT_DIR" > /dev/null

# ------------------------------ UPDATE SUBMODULES -----------------------------

git submodule update --init --recursive

# ------------------------------- LAMBDA-SH SETUP ------------------------------

source lambda-sh/lambda.sh

lambda_log_info "Successfully installed submodules and setup lambda.sh"

lambda_args_add --name within-ci \
    --default false \
    --description "Used when setup is being done within a CI system."
lambda_args_compile "$@"

if [ "$LAMBDA_within_ci" = true ]; then
    exit
fi

# ------------------------------ INSTALL LFS ASSETS ----------------------------

git lfs install > /dev/null
lambda_assert_last_command_ok "Failed to initialize git lfs"

git lfs pull > /dev/null
lambda_assert_last_command_ok "Couldn't pull LFS assets"


lambda_log_info "Updated & initialized git LFS hooks."

# ------------------------------ PRE-COMMIT SETUP ------------------------------

if command -v pre-commit > /dev/null; then
    pre-commit install > /dev/null
    lambda_log_info "Installed pre-commit hooks."
else
    lambda_log_fatal "pre-commit isn't installed. Aborting setup."
fi

lambda_log_info "Setup successfully completed."

popd > /dev/null # ROOT_DIR
