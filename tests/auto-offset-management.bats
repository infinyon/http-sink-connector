#!/usr/bin/env bats

load './bats-helpers/bats-support/load'
load './bats-helpers/bats-assert/load'
load './bats-helpers/tools_check.bash'

setup_file() {
    CDK_BIN=${CDK_BIN:-cdk}
    export CDK_BIN
    FLUVIO_BIN=${FLUVIO_BIN:-fluvio}
    export FLUVIO_BIN
    TEST_DIR="$(mktemp -d -t auto-offset-mngt-test.XXXXX)"
    export TEST_DIR
}

setup() {
    TOPIC="$(random_string)"
    export TOPIC_NAME
    CONNECTOR_NAME="my-$TOPIC"
    export CONNECTOR_NAME
    export LOG_PATH="$CONNECTOR_NAME.log"

    export LOGGER_FILENAME="${CONNECTOR_NAME}-http-server.log"
    ./target/debug/tiny-http-server & disown
    MOCK_PID=$!

    $FLUVIO_BIN topic create $TOPIC
}

@test "Read stream from the beginning" {
    CONFIG_PATH="$TEST_DIR/$TOPIC.yaml"
    cat <<EOF >$CONFIG_PATH
apiVersion: 0.2.0
meta:
  version: 0.1.0
  name: $CONNECTOR_NAME
  type: http-sink
  topic:
    meta:
      name: $TOPIC
  consumer:
    id: $CONNECTOR_NAME
    offset:
      strategy: auto
      start: beginning
http:
  endpoint: http://localhost:8080
  interval: 3s
EOF

    echo "RecordOne" | $FLUVIO_BIN produce $TOPIC
    echo "RecordTwo" | $FLUVIO_BIN produce $TOPIC

    $CDK_BIN deploy -p http-sink start --config $CONFIG_PATH --log-level info

    wait_for_line_in_file "monitoring started" $LOG_PATH 30

    wait_for_line_in_file "RecordOne" $LOGGER_FILENAME 30
    wait_for_line_in_file "RecordTwo" $LOGGER_FILENAME 30
}

@test "Consumer offset stored" {
    CONFIG_PATH="$TEST_DIR/$TOPIC.yaml"
    cat <<EOF >$CONFIG_PATH
apiVersion: 0.2.0
meta:
  version: 0.2.12
  name: $CONNECTOR_NAME
  type: http-sink
  topic:
    meta:
      name: $TOPIC
  consumer:
    id: $CONNECTOR_NAME
    offset:
      strategy: auto
http:
  endpoint: http://localhost:8080
  interval: 3s
EOF

    $CDK_BIN deploy -p http-sink start --config $CONFIG_PATH --log-level info

    wait_for_line_in_file "monitoring started" $LOG_PATH 30

    echo "RecordOne" | $FLUVIO_BIN produce $TOPIC
    sleep 15
    echo "RecordTwo" | $FLUVIO_BIN produce $TOPIC

    wait_for_line_in_file "RecordOne" $LOGGER_FILENAME 30
    wait_for_line_in_file "RecordTwo" $LOGGER_FILENAME 30

    OFFSET=$($FLUVIO_BIN consumer list -O json | jq ".[] | select(.consumer_id == \"$CONNECTOR_NAME\") | .offset")
    assert [ ! -z $OFFSET ]

}

@test "Backoff and retry on failure" {
    CONFIG_PATH="$TEST_DIR/$TOPIC.yaml"
    cat <<EOF >$CONFIG_PATH
apiVersion: 0.2.0
meta:
  version: 0.1.0
  name: $CONNECTOR_NAME
  type: http-sink
  topic:
    meta:
      name: $TOPIC
  consumer:
    id: $CONNECTOR_NAME
    offset:
      strategy: auto
      start: beginning
http:
  endpoint: http://localhost:8080
  interval: 3s
EOF

    echo "RecordOne" | $FLUVIO_BIN produce $TOPIC
    echo "RecordTwo" | $FLUVIO_BIN produce $TOPIC

    $CDK_BIN deploy -p http-sink start --config $CONFIG_PATH --log-level info

    wait_for_line_in_file "monitoring started" $LOG_PATH 30

    wait_for_line_in_file "RecordOne" $LOGGER_FILENAME 30
    wait_for_line_in_file "RecordTwo" $LOGGER_FILENAME 30

    kill $MOCK_PID

    echo "RecordThree" | $FLUVIO_BIN produce $TOPIC
    echo "RecordFour" | $FLUVIO_BIN produce $TOPIC

    sleep 20

    ./target/debug/tiny-http-server & disown
    MOCK_PID=$!

    wait_for_line_in_file "RecordThree" $LOGGER_FILENAME 30
    wait_for_line_in_file "RecordFour" $LOGGER_FILENAME 30
}

teardown() {
    $CDK_BIN deploy shutdown --name $CONNECTOR_NAME
    kill $MOCK_PID
}
