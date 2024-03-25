use std::collections::HashMap;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use reqwest::{Client, RequestBuilder};

use fluvio::Offset;
use fluvio_connector_common::{tracing, LocalBoxSink, Sink};
use crate::HttpConfig;

#[derive(Debug)]
pub(crate) struct HttpSink {
    body : Body
}
#[derive(Debug)]
struct Body{
    request: RequestBuilder,
    params: Vec<String>,
}
impl Clone for Body {
    fn clone(&self) -> Self {
        Body{
            request : self.request.try_clone().unwrap(),
            params  : self.params.clone()
        }
    }
}
impl HttpSink {
    pub(crate) fn new(config: &HttpConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(config.http_request_timeout)
            .connect_timeout(config.http_connect_timeout)
            .build()?;
        let method = config.method.parse()?;

        let mut request = client.request(method, config.endpoint.clone());
        request = request.header(reqwest::header::USER_AGENT, config.user_agent.clone());

        let headers = config.headers.iter().flat_map(|h| h.split_once(':'));
        for (key, value) in headers {
            request = request.header(key, value.trim());
        }

        Ok(Self { body: Body{request, params: config.params.clone()} })
    }
}

#[async_trait]
impl Sink<String> for HttpSink {
    async fn connect(self, _offset: Option<Offset>) -> Result<LocalBoxSink<String>> {
        let unfold = futures::sink::unfold(
            self.body,
            |mut body: Body, record: String| async move {
                let params = body.params.clone();
                if params.len() > 0 {
                    if let Ok(json_message) = serde_json::from_str::<HashMap<String, serde_json::Value>>(&record){
                        for param in params.into_iter() {
                            let (key, replace_key) = match param.split_once(':') {
                                Some(map) =>{
                                    (
                                        map.0.trim().to_owned(),
                                        map.1.trim().to_owned()
                                    )
                                }
                                None => (param.clone(), param.clone())
                            };
                            if json_message.contains_key(&key){
                                tracing::info!("Using URL parameter: {key}");
                                let value = json_message.get(&key).unwrap().to_string();
                                body.request = body.request.query(&[(replace_key, value)]);

                            }
                        }
                    }
                }
                
                tracing::trace!("{:?}", body.request);

                body.request = body.request.body(record);
                let response = body.request
                    .try_clone()
                    .ok_or(anyhow!("ERR: Cannot clone request"))?
                    .send()
                    .await?;

                if response.status().is_success() {
                    tracing::debug!("Response Status: {}", response.status());
                } else {
                    tracing::warn!("Response Status: {}", response.status());
                    tracing::debug!("{:?}", response);
                }
                Ok::<_, anyhow::Error>(body)
            },
        );

        Ok(Box::pin(unfold))
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use super::*;

    #[test]
    fn builds_http_client_from_config() {
        let config = HttpConfig {
            endpoint: "http://localhost:8080".parse().unwrap(),
            user_agent: "fluvio/http-sink 0.1.0".into(),
            method: "POST".into(),
            headers: vec!["Content-Type: text/html".into()],
            http_connect_timeout: Duration::from_secs(1),
            http_request_timeout: Duration::from_secs(15),
            params: vec![]
        };
        let sink = HttpSink::new(&config).unwrap();
        let req = sink.body.request.build().unwrap();

        assert_eq!(req.headers().get("Content-Type").unwrap(), "text/html");
        assert_eq!(
            req.headers().get("User-Agent").unwrap(),
            "fluvio/http-sink 0.1.0"
        );
        assert_eq!(req.method().to_string(), "POST");
        assert_eq!(req.url().to_string(), "http://localhost:8080/");
    }
}
