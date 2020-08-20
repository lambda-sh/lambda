#!/bin/bash
# The lambda utility script. This file is to serve as a "header" file for making
# writing bash for lambda a bit easier.

# ---------------------------- INTERNAL VARIABLES ------------------------------

export __LAMBDA_SCRIPT_REGISTERED_ARGS=()

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

# --------------------------------- TYPES ------------------------------------
LAMBDA_TYPE_NUMBER="number"
LAMBDA_TYPE_STRING="string"
LAMBDA_TYPE_LIST="list"

# --------------------------------- LOGGING ------------------------------------
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

# ------------------------------- ARG PARSING ----------------------------------

__LAMBDA_NAMESPACE_DEFAULT="LAMBDA"
__LAMBDA_NAMESPACE_CURRENT=""

__LAMBDA_GET_ACTIVE_NAMESPACE() {
    return 0;
}

LAMBDA_START_NAMESPACE() {
    __LAMBDA_CURRENT_NAMESPACE;
}


export __LAMBDA_REGISTERED_ARG_MAP=()
export __LAMBDA_ARG_DEFAULT_VALUES=()
export __LAMBDA_ARG_HELP_STRINGS=()
export __LAMBDA_ARG_IS_SET=()
export __LAMBDA_ARG_COUNT=0

# Parse an argument
# SHORT_HAND -> The short hand flag of the argument.
# LONG_HAND -> The long hand name of the argument.
# DEFAULT_VALUE -> The default value of argument.
# HELP_STRING -> The Help string for the argument.
LAMBDA_PARSE_ARG() {
    SHORT_HAND="$1"
    LONG_HAND="$2"
    DEFAULT_VALUE="$3"
    HELP_STRING="$4"

    __LAMBDA_REGISTERED_ARG_MAP+=("${SHORT_HAND}:${LONG_HAND}:${__LAMBDA_ARG_COUNT}")
    __LAMBDA_ARG_DEFAULT_VALUES+=("$DEFAULT_VALUE")
    __LAMBDA_ARG_HELP_STRINGS+=("$HELP_STRING")
    __LAMBDA_ARG_IS_SET+=(0)
    __LAMBDA_ARG_COUNT=$((1 + __LAMBDA_ARG_COUNT))
}

LAMBDA_COMPILE_ARGS() {
    if !(("$#")); then
         for ((i=0; i<$__LAMBDA_ARG_COUNT; i++)); do
            IFS=':' read -ra ARG_NAME <<< "${__LAMBDA_REGISTERED_ARG_MAP[${i}]}"

            SHORT_HAND="${ARG_NAME[0]}"
            LONG_HAND="${ARG_NAME[1]}"
            ARG_INDEX="${ARG_NAME[2]}"

            if [ "${__LAMBDA_ARG_IS_SET[${ARG_INDEX}]}" = 0 ]; then
                DEFAULT_VALUE="${__LAMBDA_ARG_DEFAULT_VALUES[${ARG_INDEX}]}"
                export "LAMBDA_${LONG_HAND}"="$DEFAULT_VALUE"
            fi
         done
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
                export "LAMBDA_${LONG_HAND}"="$DEFAULT_VALUE"
            fi

            if [ "$1" = "-$SHORT_HAND" ] || [ "$1" = "--$LONG_HAND" ]; then
                if [ -n "$2" ] && [ ${2:0:1} != "-" ]; then
                    export "LAMBDA_${LONG_HAND}"="$2"
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
