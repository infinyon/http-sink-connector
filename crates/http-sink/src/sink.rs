use std::collections::HashMap;

use crate::{config::Parameter, HttpConfig};
use anyhow::{anyhow, Result};
use fluvio::dataplane::record::ConsumerRecord;
use fluvio_connector_common::tracing;
use reqwest::{Client, RequestBuilder, Response};
use urlencoding::encode;

#[derive(Debug)]
pub(crate) struct HttpSink {
    #[allow(dead_code)]
    client: Client,
    request: RequestBuilder,
    url_parameters: Vec<Parameter>,
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

        Ok(Self {
            client,
            request,
            url_parameters: config.url_parameters.clone(),
        })
    }

    pub(crate) fn make_request(&self, record: &ConsumerRecord) -> Result<RequestBuilder> {
        let mut builder = self
            .request
            .try_clone()
            .ok_or(anyhow!("ERR: Cannot clone request"))?;

        if !self.url_parameters.is_empty() {
            let str = String::from_utf8(record.as_ref().to_vec())?;
            if let Ok(json_message) =
                serde_json::from_str::<HashMap<String, serde_json::Value>>(&str)
            {
                for param in self.url_parameters.iter() {
                    let url_key = param.url_key.clone().unwrap_or(param.record_key.clone());
                    if json_message.contains_key(&param.record_key) {
                        let mut value = json_message.get(&param.record_key).unwrap().to_string();
                        if let Some(ref prefix) = param.prefix {
                            value = prefix.clone() + &value;
                        }
                        if let Some(ref suffix) = param.suffix {
                            value = value.clone() + &suffix;
                        }
                        builder = builder.query(&[(encode(&url_key), &value)]);
                    }
                }
            }
        }
        Ok(builder)
    }

    pub(crate) async fn send(&self, record: &ConsumerRecord) -> Result<Response> {
        let str = String::from_utf8(record.as_ref().to_vec())?;

        let request = self.make_request(record)?;
        let request = request.body(str);

        let response = request.send().await?;
        if response.status().is_success() {
            tracing::debug!("Response Status: {}", response.status());
        } else {
            tracing::warn!("Response Status: {}", response.status());
            tracing::debug!("{:?}", response);
        }
        Ok(response)
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
            url_parameters: vec![],
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
