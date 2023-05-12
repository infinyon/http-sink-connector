# Fluvio HTTP Outbound Connector

Official Infinyon HTTP Sink connector

## Sink Connector

HTTP sink connector reads records from data streaming and generates an HTTP request.

> Supports HTTP/1.0, HTTP/1.1, HTTP/2.0 protocols.

### Configuration

HTTP Sink is configured using a YAML file:

```yaml
# config-example.yaml
apiVersion: 0.1.0
meta:
  version: 0.2.1
  name: my-http-sink
  type: http-sink
  topic: http-sink-topic
  secrets:
    - name: HTTP_TOKEN
http:
  endpoint: "http://127.0.0.1/post"
  headers:
    - "Authorization: token ${{ secrets.HTTP_TOKEN }}"
    - "Cache-Control: no-cache"
```

| Option               | default                    | type            | description                                       |
| :--------------------| :--------------------------| :-------------- | :-------------------------------------------------|
| method               | POST                       | String          | POST, PUT                                         |
| endpoint             | -                          | String          | HTTP URL endpoint                                 |
| headers              | -                          | Array\<String\> | Request header(s) "Key:Value" pairs               |
| user-agent           | `fluvio/http-sink 0.1.0`   | String          | Request user-agent                                |
| http_request_timeout | 1s                         | String          | HTTP Request Timeout                              |
| http_connect_timeout | 15s                        | String          | HTTP Connect Timeout                              |

> By default HTTP headers will use `Content-Type: text/html` unless anothed value
> is provided to the Headers configuration.

### Usage

Login to your Fluvio Cloud Account via Fluvio CLI

```bash
fluvio cloud login --use-oauth2
```

Then configure the HTTP Request to be sent using a YAML file following the
connector configuration schema. The following configuration will send a POST
HTTP request to `http://httpbin.org/post`.

```yaml
# config.yaml
apiVersion: 0.1.0
meta:
  version: 0.2.1
  name: httpbin
  type: http-sink
  topic: httpbin-send-post

http:
  endpoint: http://httpbin.org/post
  interval: 3s
```

Finally create your connector by running:

```bash
fluvio cloud connector create --config ./config.yaml
```

> You can see active connectors by running the following command:
>
> ```bash
> fluvio cloud connector list
> ```

Check connector logs by running

```bash
fluvio cloud connector logs httpbin
```

```log
INFO connect:connect_with_config:connect: fluvio_socket::versioned: connect to socket add=fluvio-sc-public:9003
INFO dispatcher_loop{self=MultiplexDisp(10)}: fluvio_socket::multiplexing: multiplexer terminated
2023-05-02T20:59:50.192104Z  INFO stream_with_config:inner_stream_batches_with_config:request_stream{offset=Offset { inner: FromEnd(0) }}:create_serial_socket:create_serial_socket_from_leader{leader_id=0}:connect_to_leader{leader=0}:connect: fluvio_socket::versioned: connect to socket add=fluvio-spu-main-0.acct-584fd564-1d4a-4308-9061-09acea387bea.svc.cluster.local:9005
INFO fluvio_connector_common::monitoring: using metric path: /fluvio_metrics/connector.sock
INFO fluvio_connector_common::monitoring: monitoring started
```

### Produce Records to send as HTTP POST Requests

You can produce records using `fluvio produce <TOPIC>`, values produced will
be sent as HTTP Body payloads on HTTP Sink Connector.

Running the following command will attach stdin to the topic stream, any data
written to stdin will be sent as a record through the `httpbin-send-post` topic,
and as a side effect of the HTTP Sink Connector, these records will also be sent
as HTTP POST requests to http://httpbin.org/post, based on our configuration.

```bash
fluvio produce httpbin-send-post
```

Then send data:

```log
> {\"hello\": \"world\"}
Ok!
```

### Teardown

To stop your connector just use `fluvio cloud connector delete <NAME>`

```bash
fluvio cloud connector delete httpbin
```

> `httpbin` is our connector instance name from the configuration file shown above

### Transformations

Fluvio HTTP Sink Connector supports [Transformations](https://www.fluvio.io/docs/concepts/transformations-chain/). Records can be modified before sending to endpoint.

The previous example can be extended to add extra transformations to outgoing records:
```yaml
# config-example.yaml
apiVersion: 0.1.0
meta:
  version: 0.2.1
  name: my-http-sink
  type: http-sink
  topic: http-sink-topic
  secrets:
    - name: AUTHORIZATION_TOKEN
http:
  endpoint: "http://127.0.0.1/post"
  headers:
    - "Authorization: token ${{ secrets.AUTHORIZATION_TOKEN }}"
    - "Content-Type: application/json"
transforms:
  - uses: infinyon/jolt@0.1.0
    with:
      spec:
        - operation: shift
          spec:
            "result": "text"
```

In this case, additional transformation will be performed before records are sent the http endpoint. A json field called `result` will be renamed to `text`.


Read more about [JSON to JSON transformations](https://www.fluvio.io/smartmodules/certified/jolt/).

## Contributing

Follow on the conventional `CONTRIBUTING.md` file to setup your environment and
contribute to this project.

[1]: https://www.fluvio.io/connectors/outbound/http/
