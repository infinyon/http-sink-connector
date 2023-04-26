#!/usr/bin/env bats

load './bats-helpers/bats-support/load'
load './bats-helpers/bats-assert/load'

setup() {
    UUID=$(uuidgen | awk '{print tolower($0)}')
    TOPIC=${UUID}-topic

    export LOGGER_FILENAME="${UUID}-logs.txt"
    ./target/debug/tiny-http-server & disown
    MOCK_PID=$!

    cdk publish --pack -p http-sink

    FILE=$(mktemp)
    cp ./tests/integration-sends-data-via-post.yaml $FILE

    CONNECTOR=${UUID}-sends-data
    VERSION=$(cat ./crates/http-sink/hub/package-meta.yaml | grep "^version:" | cut -d" " -f2)
    IPKG_NAME="http-sink-$VERSION.ipkg"
    fluvio topic create $TOPIC

    sed -i.BAK "s/CONNECTOR/${CONNECTOR}/g" $FILE
    sed -i.BAK "s/TOPIC/${TOPIC}/g" $FILE
    sed -i.BAK "s/VERSION/${VERSION}/g" $FILE
    cat $FILE

    cdk deploy start --ipkg ./crates/http-sink/hub/$IPKG_NAME --config $FILE
}

teardown() {
    fluvio topic delete $TOPIC
    cdk deploy shutdown $CONNECTOR
    kill $MOCK_PID
}

@test "integration-sends-data-via-post" {
    echo "Starting consumer on topic $TOPIC"
    echo "Using connector $CONNECTOR"
    sleep 45

    echo "Produce \"California\" on $TOPIC"
    echo "California" | fluvio produce $TOPIC

    echo "Sleep to ensure record is processed"
    sleep 25

    echo "Contains California on Logger File"
    cat ./$LOGGER_FILENAME | grep "California"
    assert_success
}
