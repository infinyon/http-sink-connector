# HTTP Sink Connector

Official Infinyon HTTP Sink connector

## Sink Connector

HTTP sink connector reads records from data streaming and generates an HTTP  

Supports HTTP/1.0, HTTP/1.1, HTTP/2.0 protocols.


See [docs](#) here.

### Configuration

| Option       | default                    | type            | description                                       |
| :------------| :--------------------------| :-------------- | :-------------------------------------------------|
| method       | POST                       | String          | POST, PUT                                         |
| endpoint     | -                          | String          | HTTP URL endpoint                                 |
| headers      | -                          | Array\<String\> | Request header(s) "Key:Value" pairs               |
| user-agent   | "fluvio/http-sink 0.1.0"   | String          | Request user-agent                                |

> By default HTTP headers will use `Content-Type: text/html` unless anothed value
> is provided to the Headers configuration.

### Usage Example

This is an example of simple connector config file:

```yaml
# config-example.yaml
meta:
  version: 0.1.0
  name: my-http-sink
  type: http-sink
  topic: http-sink-topic
http:
  endpoint: "http://127.0.0.1/post"
  headers:
    - "Authorization: token MySecretToken"
    - "Cache-Control: no-cache"
```


### Transformations

Fluvio HTTP Sink Connector supports [Transformations](https://www.fluvio.io/docs/concepts/transformations-chain/). Records can be modified before sending to endpoint.

The previous example can be extended to add extra transformations to outgoing records:
```yaml
# config-example.yaml
meta:
  version: 0.1.0
  name: my-http-sink
  type: http-sink
  topic: http-sink-topic
http:
  endpoint: "http://127.0.0.1/post"
  headers:
    - "Authorization: token MySecretToken"
    - "Cache-Control: no-cache"
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
