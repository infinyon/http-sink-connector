use std::time::Duration;

use fluvio_connector_common::connector;
use serde::Deserialize;
use url::Url;

const DEFAULT_USER_AGENT: &str = concat!("fluvio/http-sink ", env!("CARGO_PKG_VERSION"));
const DEFAULT_HTTP_METHOD: &str = "POST";
const DEFAULT_HTTP_HEADERS: [&str; 1] = ["Content-Type: text/html"];

#[derive(Debug)]
#[connector(config, name = "http")]
pub(crate) struct HttpConfig {
    /// Target HTTP service endpoint
    pub endpoint: Url,

    /// HTTP user-agent header for the request
    #[serde(default = "default_user_agent")]
    pub user_agent: String,

    /// HTTP method used in the request POST, PUT, etc.
    #[serde(default = "default_http_method")]
    pub method: String,

    /// Headers to include in the HTTP request, in "Key=Value" format
    #[serde(default = "default_http_headers")]
    pub headers: Vec<String>,

    /// Http request timeout in seconds
    #[serde(with = "humantime_serde", default = "default_http_timeout")]
    pub http_request_timeout: Duration,

    /// Http connect timeout in milliseconds
    #[serde(with = "humantime_serde", default = "default_http_connect_timeout")]
    pub http_connect_timeout: Duration,

    //HTTP Parameters that can be gattered from a Message if the message is a json file
    #[serde(default = "default_http_params")]
    pub url_parameters: Vec<Parameter>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Parameter{
    /// The key that will be get from 
    pub record_key: String,
    pub url_key: Option<String>,
    pub prefix: Option<String>,
    pub suffix: Option<String>
}

#[inline]
fn default_http_timeout() -> Duration {
    Duration::from_secs(15)
}

#[inline]
fn default_http_connect_timeout() -> Duration {
    Duration::from_secs(1)
}

#[inline]
fn default_user_agent() -> String {
    DEFAULT_USER_AGENT.into()
}

#[inline]
fn default_http_method() -> String {
    DEFAULT_HTTP_METHOD.into()
}

#[inline]
fn default_http_headers() -> Vec<String> {
    DEFAULT_HTTP_HEADERS.map(String::from).into_iter().collect()
}

fn default_http_params () -> Vec<Parameter>{
    vec![]
}