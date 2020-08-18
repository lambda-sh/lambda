#!/bin/bash
# The lambda utility script. This file is to serve as a "header" file for making
# writing bash for lambda a bit easier.

# ---------------------------- INTERNAL VARIABLES ------------------------------

__LAMBDA_SCRIPT_REGISTERED_PARAMS=()

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

# ---------------------------- INTERNAL FUNCTIONS ------------------------------

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

__LAMBDA_CLEAR_FOREGROUND() {
    tput setaf $__LAMBDA_COLOR_NOT_USED
}

__LAMBDA_SET_BACKGROUND() {
    tput setab $1
}

__LAMBDA_CLEAR_BACKGROUND() {
    tput setab $__LAMBDA_COLOR_NOT_USED
}

__LAMBDA_CLEAR_SCREEN() {
    tput clear
}

# ---------------------------- EXTERNAL VARIABLES ------------------------------

LAMBDA_NUMBER_TYPE="number"
LAMBDA_STRING_TYPE="string"
LAMBDA_LIST_TYPE="list"

# ---------------------------- EXTERNAL FUNCTIONS ------------------------------

LAMBDA_TRACE() {
    __LAMBDA_SET_FOREGROUND $__LAMBDA_COLOR_WHITE
    __LAMBDA_SET_BACKGROUND $__LAMBDA_COLOR_BLACK
    __LAMBDA_SET_BOLD
    printf "[TRACE][%s][%s]:" $(date +"%F") $(date +"%T")
    __LAMBDA_CLEAR_ATTRIBUTES
    printf " $1\n"
}

# Log info to the console.
LAMBDA_INFO() {
    __LAMBDA_SET_FOREGROUND $__LAMBDA_COLOR_WHITE
    __LAMBDA_SET_BACKGROUND $__LAMBDA_COLOR_GREEN
    __LAMBDA_SET_BOLD
    printf "[INFO][%s][%s]:" $(date +"%F") $(date +"%T")
    __LAMBDA_CLEAR_ATTRIBUTES
    printf " $1\n"
}

LAMBDA_WARN() {
    __LAMBDA_SET_FOREGROUND $__LAMBDA_COLOR_WHITE
    __LAMBDA_SET_BACKGROUND $__LAMBDA_COLOR_YELLOW
    __LAMBDA_SET_BOLD
    printf "[WARN][%s][%s]:" $(date +"%F") $(date +"%T")
    __LAMBDA_CLEAR_ATTRIBUTES
    printf " $1\n"
}

LAMBDA_ERROR() {
    __LAMBDA_SET_FOREGROUND $__LAMBDA_COLOR_WHITE
    __LAMBDA_SET_BACKGROUND $__LAMBDA_COLOR_RED
    __LAMBDA_SET_BOLD
    printf "[ERROR][%s][%s]:" $(date +"%F") $(date +"%T")
    __LAMBDA_CLEAR_ATTRIBUTES
    printf " $1\n"
}

LAMBDA_FATAL() {
    __LAMBDA_SET_FOREGROUND $__LAMBDA_COLOR_BLACK
    __LAMBDA_SET_BACKGROUND $__LAMBDA_COLOR_RED
    __LAMBDA_SET_BOLD
    printf "[FATAL][%s][%s]:" $(date +"%F") $(date +"%T")
    __LAMBDA_CLEAR_ATTRIBUTES
    printf " $1\n"
    exit 1
}

# Parse an argument
# SHORT_HAND -> The short hand flag of the argument.
# LONG_HAND -> The long hand name of the argument.
# LAMBDA_TYPE -> The type of value being stored.
# DEFAULT_VALUE -> The default value of argument.
# HELP_STRING -> The Help string for the argument.
LAMBDA_PARSE_ARG() {
    SHORT_HAND="$1"
    LONG_HAND="$2"
    LAMBDA_TYPE="$3"
    DEFAULT_VALUE="$4"
    HELP_STRING="$5"

    REGISTERED_PARAMS+=
        ($SHORT_HAND, $LONG_HAND, $LAMBDA_TYPE ,$DEFAULT_VALUE, $HELP_STRING)
}

LAMBDA_COMPILE_ARGS() {
    echo "Not yet implemented"
}
