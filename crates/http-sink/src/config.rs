use fluvio_connector_common::connector;
use url::Url;

const DEFAULT_USER_AGENT: &str = "fluvio/http-sink 0.1.0";
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
    #[serde(default = "default_http_timeout_secs")]
    pub http_timeout_secs: u64,

    /// Http connect timeout in milliseconds
    #[serde(default = "default_http_connect_timeout_millis")]
    pub http_connect_timeout_millis: u64,
}

#[inline]
fn default_http_timeout_secs() -> u64 {
    15
}

#[inline]
fn default_http_connect_timeout_millis() -> u64 {
    1000
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
