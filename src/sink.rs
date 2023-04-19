use anyhow::{anyhow, Result};
use async_trait::async_trait;
use reqwest::{Client, RequestBuilder};

use fluvio::Offset;
use fluvio_connector_common::{tracing::trace, LocalBoxSink, Sink};

use crate::HttpConfig;

#[derive(Debug)]
pub(crate) struct HttpSink {
    request: RequestBuilder,
}

impl HttpSink {
    pub(crate) fn new(config: &HttpConfig) -> Result<Self> {
        let client = Client::new();
        let method = config.method.parse()?;

        let mut request = client.request(method, config.endpoint.clone());
        request = request.header(reqwest::header::USER_AGENT, config.user_agent.clone());

        let headers = config.headers.iter().flat_map(|h| h.split_once(':'));
        for (key, value) in headers {
            request = request.header(key, value.trim());
        }

        Ok(Self { request })
    }
}

#[async_trait]
impl Sink<String> for HttpSink {
    async fn connect(self, _offset: Option<Offset>) -> Result<LocalBoxSink<String>> {
        let request = self.request;
        let unfold = futures::sink::unfold(
            request,
            |mut request: RequestBuilder, record: String| async move {
                trace!("{:?}", request);

                request = request.body(record);
                request
                    .try_clone()
                    .ok_or(anyhow!("ERR: Cannot clone request"))?
                    .send()
                    .await?;

                Ok::<_, anyhow::Error>(request)
            },
        );

        Ok(Box::pin(unfold))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn builds_http_client_from_config() {
        let config = HttpConfig {
            endpoint: "http://localhost:8080".parse().unwrap(),
            user_agent: "fluvio/http-sink 0.1.0".into(),
            method: "POST".into(),
            headers: vec!["Content-Type: text/html".into()],
        };
        let sink = HttpSink::new(&config).unwrap();
        let req = sink.request.build().unwrap();

        assert_eq!(req.headers().get("Content-Type").unwrap(), "text/html");
        assert_eq!(
            req.headers().get("User-Agent").unwrap(),
            "fluvio/http-sink 0.1.0"
        );
        assert_eq!(req.method().to_string(), "POST");
        assert_eq!(req.url().to_string(), "http://localhost:8080/");
    }
}
