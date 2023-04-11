use anyhow::{anyhow, Result};
use async_trait::async_trait;
use reqwest::{Client, RequestBuilder};
use fluvio::Offset;
use fluvio_connector_common::{
    Sink, LocalBoxSink,
    tracing::trace
};

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
      
      let headers = 
        config.headers
            .iter()
            .flat_map(|h| h.split_once(':'));
      for (key, value) in headers {
          request = request.header(key, value);
      }

      Ok(Self {
          request,
      })
    }
}

#[async_trait]
impl Sink<String> for HttpSink {
    async fn connect(self, _offset: Option<Offset>) -> Result<LocalBoxSink<String>> {
        let request = self.request;

        let unfold = futures::sink::unfold(
            request,
            | mut request: RequestBuilder, record: String| async move {
                trace!("{:?}", request);

                request = request.body(record);
                request.try_clone().ok_or(anyhow!("ERR: Cannot clone request"))?.send().await?;

                Ok::<_, anyhow::Error>(request)
            },
        );
        Ok(Box::pin(unfold))
    }
}
