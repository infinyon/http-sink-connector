function wait_for_line_in_file() {
    LINE="$1"
    FILE="$2"
    MAX_SECONDS="${3:-30}" # 30 seconds default value

    echo "Waiting for file $FILE containing $LINE"
  
    ELAPSED=0;
    until grep -q "$LINE" "$FILE"
    do
      sleep 1
      let ELAPSED=$ELAPSED+1
      if [[ $ELAPSED -ge MAX_SECONDS ]]
      then
        echo "timeout $MAX_SECONDS seconds elapsed"
        exit 1
      fi  
    done
    echo "Done waiting for file $FILE containing $LINE"
}

function random_string() {
    DEFAULT_LEN=7
    STRING_LEN=${1:-$DEFAULT_LEN}

    # Ensure we don't end up with a string that starts or ends with '-'
    # https://github.com/infinyon/fluvio/issues/1864

    HEAD=$(shuf -zer -n1 {a..z} | tr -d '\0')
    BODY=$(shuf -zer -n"$(($STRING_LEN - 2))" {a..z} {0..9} "-" | tr -d '\0')
    FOOT=$(shuf -zer -n1 {a..z} {0..9} | tr -d '\0')

    RANDOM_STRING=$HEAD$BODY$FOOT

    if [[ -n $DEBUG ]]; then
        echo "# DEBUG: Random str (len: $STRING_LEN): $RANDOM_STRING" >&3
    fi

    export RANDOM_STRING
    echo "$RANDOM_STRING"
}
