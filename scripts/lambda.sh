#!/bin/bash
# The lambda utility script. This file is to serve as a "header" file for making
# writing bash scripts bit easier. This has been built and tested with
# bash 5.0.18
#
# Some notes about this file:
# * Variables and functions that match __LAMBDA* should not be used externally.
# * This is still highly experimental and hasn't been used/tested across
#   multiple shells and platforms.
# * the compile_and_run.sh script holds very good example usage and is integral
#   in the script itself.

# ---------------------------------- COLORS ------------------------------------

__LAMBDA_COLOR_BLACK=0
__LAMBDA_COLOR_RED=1
__LAMBDA_COLOR_GREEN=2
__LAMBDA_COLOR_YELLOW=3
__LAMBDA_COLOR_BLUE=4
__LAMBDA_COLOR_MAGENTA=5
__LAMBDA_COLOR_CYAN=6
__LAMBDA_COLOR_WHITE=7
__LAMBDA_COLOR_NOT_USED=8
__LAMBDA_COLOR_DEFAULT=9

# Make output bold.
__LAMBDA_SET_BOLD() {
    tput bold
}

# Make output underlined.
__LAMBDA_SET_UNDERLINE() {
    tput smul
}

# Make the output blink.
__LAMBDA_SET_BLINK() {
    tput blink
}

# Make the output standout.
__LAMBDA_SET_STANDOUT() {
    tput blink
}

# Clear all attributes.
__LAMBDA_CLEAR_ATTRIBUTES() {
    tput sgr0
}

# Set the foreground color.
__LAMBDA_SET_FOREGROUND() {
    tput setaf $1
}

# Reset the foreground to it's default.
__LAMBDA_CLEAR_FOREGROUND() {
    tput setaf $__LAMBDA_COLOR_DEFAULT
}

# Set the background color.
__LAMBDA_SET_BACKGROUND() {
    tput setab $1
}

# Clear the background.
__LAMBDA_CLEAR_BACKGROUND() {
    tput setab $__LAMBDA_COLOR_NOT_USED
}

# Clear the entire screen.
__LAMBDA_CLEAR_SCREEN() {
    tput clear
}

# --------------------------------- TYPES ------------------------------------

LAMBDA_TYPE_NUMBER="number"
LAMBDA_TYPE_STRING="string"
LAMBDA_TYPE_LIST="list"

# --------------------------------- LOGGING ------------------------------------

# Log a trace/debug message.
# Example usage:
# LAMBDA_TRACE "This is an trace message."j
LAMBDA_TRACE() {
    __LAMBDA_SET_FOREGROUND $__LAMBDA_COLOR_WHITE
    __LAMBDA_SET_BACKGROUND $__LAMBDA_COLOR_BLACK
    __LAMBDA_SET_BOLD
    printf "[TRACE][%s][%s]:" $(date +"%F") $(date +"%T")
    __LAMBDA_CLEAR_ATTRIBUTES
    printf " $1\n"
}

# Log an info message.
# Example usage:
# LAMBDA_WARN "This is an informational message."j
LAMBDA_INFO() {
    __LAMBDA_SET_FOREGROUND $__LAMBDA_COLOR_BLACK
    __LAMBDA_SET_BACKGROUND $__LAMBDA_COLOR_GREEN
    __LAMBDA_SET_BOLD
    printf "[INFO][%s][%s]:" $(date +"%F") $(date +"%T")
    __LAMBDA_CLEAR_ATTRIBUTES
    printf " $1\n"
}

# Log an error message.
# Example usage:
# LAMBDA_WARN "This is a warning message."j
LAMBDA_WARN() {
    __LAMBDA_SET_FOREGROUND $__LAMBDA_COLOR_BLACK
    __LAMBDA_SET_BACKGROUND $__LAMBDA_COLOR_YELLOW
    __LAMBDA_SET_BOLD
    printf "[WARN][%s][%s]:" $(date +"%F") $(date +"%T")
    __LAMBDA_CLEAR_ATTRIBUTES
    printf " $1\n"
}

# Log an error message.
# Example usage:
# LAMBDA_ERROR "This is an error message."j
LAMBDA_ERROR() {
    __LAMBDA_SET_FOREGROUND $__LAMBDA_COLOR_BLACK
    __LAMBDA_SET_BACKGROUND $__LAMBDA_COLOR_RED
    __LAMBDA_SET_BOLD
    printf "[ERROR][%s][%s]:" $(date +"%F") $(date +"%T")
    __LAMBDA_CLEAR_ATTRIBUTES
    printf " $1\n"
}

# Log a fatal message and quit.
# Example usage:
# LAMBDA_FATAL "Couldn't load a file, exiting the script."
LAMBDA_FATAL() {
    __LAMBDA_SET_FOREGROUND $__LAMBDA_COLOR_BLACK
    __LAMBDA_SET_BACKGROUND $__LAMBDA_COLOR_RED
    __LAMBDA_SET_BOLD
    printf "[FATAL][%s][%s]:" $(date +"%F") $(date +"%T")
    __LAMBDA_CLEAR_ATTRIBUTES
    printf " $1\n"
    exit 1
}

# -------------------------------- ARG PARSING ---------------------------------

export __LAMBDA_REGISTERED_ARG_MAP=()
export __LAMBDA_ARG_DEFAULT_VALUES=()
export __LAMBDA_ARG_HELP_STRINGS=()
export __LAMBDA_ARG_IS_SET=()
export __LAMBDA_ARG_COUNT=0

# Parse an argument that you want to use for your script.
# Example usage looks like:
# LAMBDA_PARSE_ARG tool sandbox "The tool to compile and run."
#
# This would register an argument given the arg name (--tool),
# default value (sandbox), and lastly a help string. The arguments are then
# pushed into arrays and bounded by a key to the index they hold within all
# arrays.
#
# ARG_NAME -> The long hand name of the argument.
# DEFAULT_VALUE -> The default value of argument.
# HELP_STRING -> The Help string for the argument.
LAMBDA_PARSE_ARG() {
    ARG_NAME="$1"
    DEFAULT_VALUE="$2"
    DESCRIPTION="$3"

    ARG_NAME_TO_INDEX="${ARG_NAME}:${__LAMBDA_ARG_COUNT}"
    __LAMBDA_REGISTERED_ARG_MAP+=("$ARG_NAME_TO_INDEX")
    __LAMBDA_ARG_DEFAULT_VALUES+=("$DEFAULT_VALUE")
    __LAMBDA_ARG_DESCRIPTIONS+=("$DESCRIPTION")
    __LAMBDA_ARG_IS_SET+=(0)
    __LAMBDA_ARG_COUNT=$((1 + __LAMBDA_ARG_COUNT))
}

__LAMBDA_SHOW_HELP_STRING() {
  __LAMBDA_SET_FOREGROUND $__LAMBDA_COLOR_GREEN
  printf "\n%-20s %-20s %-20s %-20s\n" "Arg" "Default value" "Required" "Description"
  __LAMBDA_CLEAR_ATTRIBUTES

  for ((i=0; i<$__LAMBDA_ARG_COUNT; i++)); do
    IFS=':' read -ra ARG_MAP <<< "${__LAMBDA_REGISTERED_ARG_MAP[${i}]}"

    ARG_NAME="${ARG_MAP[0]}"
    ARG_INDEX="${ARG_MAP[1]}"
    ARG_DEFAULT_VALUE="${__LAMBDA_ARG_DEFAULT_VALUES[${ARG_INDEX}]}"
    ARG_DESCRIPTION="${__LAMBDA_ARG_DESCRIPTIONS[${ARG_INDEX}]}"
    ARG_REQUIRED="False"

    if [ -z "${__LAMBDA_ARG_DEFAULT_VALUES[${ARG_INDEX}]}" ]; then
      ARG_REQUIRED="True"
    fi

    printf "%-20s %-20s %-20s %-20s\n" \
      "--$ARG_NAME" \
      "$ARG_DEFAULT_VALUE" \
      "$ARG_REQUIRED" \
      "$ARG_DESCRIPTION"

  done
  printf "\n"
}

# Compile a list of arguments that are created from LAMBDA_PARSE_ARG calls.
# Example usage:
#
# LAMBDA_PARSE_ARG tool sandbox "This is an argument help string"
# LAMBDA_COMPILE_ARGS $@
# echo $LAMBDA_tool
#
# You first register the arguments that you want your script to take in and then
# forward your scripts arguments directly into LAMBDA_COMPILE_ARGS. If nothing
# goes wrong with parsing then you'll have access to either the value or default
# value of the argument that you've passed in as shown with $LAMBDA_tool.
#
# Values that have no default values are assumed to be arguments that are
# required to be passed in by the person interacting with the script.
#
# TODO(C3NZ): This can be broken down into sub components and also has
# repetitive & potentially inefficient behaviour. While this isn't problematic
# right now, this implementation might not be concrete depending on finding an
# implementation that works better than using multiple arrays.
LAMBDA_COMPILE_ARGS() {
  if [ "$1" = "--help" ]; then
    __LAMBDA_SHOW_HELP_STRING $0
    LAMBDA_FATAL "Script execution disabled when using --help"
  fi

  # Iterate through the arguments and parse them into variables.
  while (("$#")); do
    FOUND=0
    for ((i=0; i<$__LAMBDA_ARG_COUNT; i++)); do
        IFS=':' read -ra ARG_MAP <<< "${__LAMBDA_REGISTERED_ARG_MAP[${i}]}"

        ARG_NAME="${ARG_MAP[0]}"
        ARG_INDEX="${ARG_MAP[1]}"

        if [ "$1" = "--$ARG_NAME" ]; then
            if [ -n "$2" ] && [ ${2:0:1} != "-" ]; then
                export "LAMBDA_${ARG_NAME//-/_}"="$2"
                __LAMBDA_ARG_IS_SET[${ARG_INDEX}]=1
                FOUND=1
                shift 2
                break
            else
                LAMBDA_FATAL "No argument for flag $1"
            fi
        fi
     done

     # If the argument cannot be found, let the user know that
     # it's not an unsupported flag or a positional argument.
     if [ $FOUND = 0 ]; then
        if [[ "$1" =~ --* ]]; then
            LAMBDA_FATAL \
              "Unsupported flag: $1. Run with --help to see the flags."
        else
            LAMBDA_FATAL "No support for positional arguments."
        fi
     fi
  done

  # Add default values to any argument that wasn't given a value.
  for ((i=0; i<$__LAMBDA_ARG_COUNT; i++)); do
    IFS=':' read -ra ARG_MAP <<< "${__LAMBDA_REGISTERED_ARG_MAP[${i}]}"

    ARG_NAME="${ARG_MAP[0]}"
    ARG_INDEX="${ARG_MAP[1]}"

    if [ "${__LAMBDA_ARG_IS_SET[${ARG_INDEX}]}" = 0 ]; then
      if [ -z "${__LAMBDA_ARG_DEFAULT_VALUES[${ARG_INDEX}]}" ]; then
        LAMBDA_FATAL \
          "--$ARG_NAME has no default value and therefore cannot be left empty."
      fi
        DEFAULT_VALUE="${__LAMBDA_ARG_DEFAULT_VALUES[${ARG_INDEX}]}"
        export "LAMBDA_${ARG_NAME//-/_}"="$DEFAULT_VALUE"
    fi
  done
}
