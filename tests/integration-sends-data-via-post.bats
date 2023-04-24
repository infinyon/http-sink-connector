#!/usr/bin/env bats

load './bats-helpers/bats-support/load'
load './bats-helpers/bats-assert/load'

setup() {
    UUID=$(uuidgen | awk '{print tolower($0)}')
    TOPIC=${UUID}-topic

    export LOGGER_FILENAME="${UUID}-logs.txt"

    ./target/debug/tiny-http-server & disown
    MOCK_PID=$!

    FILE=$(mktemp)
    cp ./tests/integration-sends-data-via-post.yaml $FILE
    CONNECTOR=${UUID}-sends-data
    VERSION=$(cat ./crates/http-sink/hub/package-meta.yaml | grep "^version:" | cut -d" " -f2)

    sed -i.BAK "s/CONNECTOR/${CONNECTOR}/g" $FILE
    sed -i.BAK "s/TOPIC/${TOPIC}/g" $FILE
    sed -i.BAK "s/VERSION/${VERSION}/g" $FILE
    cat $FILE

    fluvio topic create $TOPIC
    cdk test -p http-sink -c $FILE & disown
    CONNECTOR_PID=$!
}

teardown() {
    kill $MOCK_PID
    kill $CONNECTOR_PID
}

@test "integration-sends-data-via-post" {
    echo "Starting consumer on topic $TOPIC"
    echo "Using connector $CONNECTOR"
    sleep 45

    echo "Produce \"California\" on $TOPIC"
    echo "California" | fluvio produce $TOPIC

    echo "Contains California on Logger File"
    cat ./$LOGGER_FILENAME | grep "California"
    assert_success
}
