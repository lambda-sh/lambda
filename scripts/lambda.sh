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
# LAMBDA_PARSE_ARG t tool sandbox "The tool to compile and run."
#
# This would register an argument given the short hand (-t), long hand (--tool),
# default value (sandbox), and lastly a help string. The arguments are then
# pushed into arrays and bounded by a key to the index they hold within all
# arrays.
#
# SHORT_HAND -> The short hand flag of the argument.
# LONG_HAND -> The long hand name of the argument.
# DEFAULT_VALUE -> The default value of argument.
# HELP_STRING -> The Help string for the argument.
LAMBDA_PARSE_ARG() {
    SHORT_HAND="$1"
    LONG_HAND="$2"
    DEFAULT_VALUE="$3"
    HELP_STRING="$4"

    MAP_KEY="${SHORT_HAND}:${LONG_HAND}:${__LAMBDA_ARG_COUNT}"
    __LAMBDA_REGISTERED_ARG_MAP+=("$MAP_KEY")
    __LAMBDA_ARG_DEFAULT_VALUES+=("$DEFAULT_VALUE")
    __LAMBDA_ARG_HELP_STRINGS+=("$HELP_STRING")
    __LAMBDA_ARG_IS_SET+=(0)
    __LAMBDA_ARG_COUNT=$((1 + __LAMBDA_ARG_COUNT))
}

# Compile a list of arguments that are created from LAMBDA_PARSE_ARG calls.
# Example usage:
#
# LAMBDA_PARSE_ARG t tool sandbox "This is an argument help string"
# LAMBDA_COMPILE_ARGS $@
# echo $LAMBDA_tool
#
# You first register the arguments that you want your script to take in and then
# forward your scripts arguments directly into LAMBDA_COMPILE_ARGS. If nothing
# goes wrong with parsing then you'll have access to either the value or default
# value of the argument that you've passed in as shown with $LAMBDA_tool.
#
# TODO(C3NZ): This can be broken down into sub components and also has
# repetitive & potentially inefficient behaviour. While this isn't problematic
# right now, this implementation might not be concrete depending on finding an
# implementation that works better than using multiple arrays.
LAMBDA_COMPILE_ARGS() {
    if !(("$#")); then
         for ((i=0; i<$__LAMBDA_ARG_COUNT; i++)); do
            IFS=':' read -ra ARG_NAME <<< "${__LAMBDA_REGISTERED_ARG_MAP[${i}]}"

            SHORT_HAND="${ARG_NAME[0]}"
            LONG_HAND="${ARG_NAME[1]}"
            ARG_INDEX="${ARG_NAME[2]}"

            if [ "${__LAMBDA_ARG_IS_SET[${ARG_INDEX}]}" = 0 ]; then
                DEFAULT_VALUE="${__LAMBDA_ARG_DEFAULT_VALUES[${ARG_INDEX}]}"
                export "LAMBDA_${LONG_HAND//-/_}"="$DEFAULT_VALUE"
            fi
         done
         return
    fi

    while (("$#")); do
        FOUND=0
        for ((i=0; i<$__LAMBDA_ARG_COUNT; i++)); do
            IFS=':' read -ra ARG_NAME <<< "${__LAMBDA_REGISTERED_ARG_MAP[${i}]}"

            SHORT_HAND="${ARG_NAME[0]}"
            LONG_HAND="${ARG_NAME[1]}"
            ARG_INDEX="${ARG_NAME[2]}"

            if [ "${__LAMBDA_ARG_IS_SET[${ARG_INDEX}]}" = 0 ]; then
                DEFAULT_VALUE="${__LAMBDA_ARG_DEFAULT_VALUES[${ARG_INDEX}]}"
                export "LAMBDA_${LONG_HAND//-/_}"="$DEFAULT_VALUE"
            fi

            if [ "$1" = "-$SHORT_HAND" ] || [ "$1" = "--$LONG_HAND" ]; then
                if [ -n "$2" ] && [ ${2:0:1} != "-" ]; then
                    export "LAMBDA_${LONG_HAND//-/_}"="$2"
                    __LAMBDA_ARG_IS_SET[${ARG_INDEX}]=1
                    FOUND=1
                    echo $FOUND
                    shift 2
                    break
                else
                    LAMBDA_FATAL "No argument for flag $1"
                fi
            fi
         done
         # Check to see if the argument has been found.
         if [[ $FOUND = 0 ]]; then
            if [ "$1" = "-*" ] || [ "$1" = "--*" ]; then
                LAMBDA_FATAL "Unsupported flag $1"
            else
                LAMBDA_FATAL "No support for positional arguments."
            fi
         fi
    done
}
