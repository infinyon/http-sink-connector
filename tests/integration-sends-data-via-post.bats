#!/usr/bin/env bats

load './bats-helpers/bats-support/load'
load './bats-helpers/bats-assert/load'
load './bats-helpers/tools_check.bash'

setup() {
    UUID=$(uuidgen | awk '{print tolower($0)}')
    TOPIC=${UUID}-topic

    export LOGGER_FILENAME="${UUID}-logs.txt"
    ./target/debug/tiny-http-server & disown
    MOCK_PID=$!

    FILE=$(mktemp)
    cp ./tests/integration-sends-data-via-post.yaml $FILE

    CONNECTOR=${UUID}-sends-data
    LOG_PATH="$CONNECTOR.log"
    export VERSION=$(cat ./crates/http-sink/Connector.toml | grep "^version = " | cut -d"\"" -f2)
    IPKG_NAME="http-sink-$VERSION.ipkg"
    fluvio topic create $TOPIC

    sed -i.BAK "s/CONNECTOR/${CONNECTOR}/g" $FILE
    sed -i.BAK "s/TOPIC/${TOPIC}/g" $FILE
    sed -i.BAK "s/VERSION/${VERSION}/g" $FILE
    cat $FILE

    cdk deploy -p http-sink start --config $FILE --log-level info
}

teardown() {
    cdk deploy shutdown --name $CONNECTOR
    kill $MOCK_PID
}

@test "integration-sends-data-via-post" {
    echo "Starting consumer on topic $TOPIC"
    echo "Using connector $CONNECTOR"

    wait_for_line_in_file "monitoring started" $LOG_PATH 30

    echo "Produce \"California\" on $TOPIC"
    echo "California" | fluvio produce $TOPIC

    echo "Contains California on Logger File"
    wait_for_line_in_file "California" $LOGGER_FILENAME 30

    assert_success
}

@test "sends-user-agent-with-current-version" {
    echo "Starting consumer on topic $TOPIC"
    echo "Using connector $CONNECTOR"

    wait_for_line_in_file "monitoring started" $LOG_PATH 30

    echo "Produce \"North Carolina\" on $TOPIC"
    echo "North Carolina" | fluvio produce $TOPIC

    echo "Contains User Agent with current version"
    wait_for_line_in_file "user_agent" $LOGGER_FILENAME 30
    cat ./$LOGGER_FILENAME | grep "user_agent: \"fluvio/http-sink $VERSION\""

    assert_success
}
