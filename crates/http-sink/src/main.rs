mod config;
mod sink;

use anyhow::Result;
use config::HttpConfig;
use sink::HttpSink;

use fluvio_connector_common::{connector, consumer::ConsumerStream, tracing};

const SIGNATURES: &str = concat!("InfinyOn HTTP Sink Connector ", env!("CARGO_PKG_VERSION"));

#[connector(sink)]
async fn start(config: HttpConfig, mut stream: impl ConsumerStream) -> Result<()> {
    tracing::debug!(?config);
    tracing::info!("Starting {SIGNATURES}");
    let sink = HttpSink::new(&config)?;
    while let Some(item) = stream.next().await {
        tracing::debug!("Received record in consumer");
        let rec = item?;
        sink.send(&rec).await?;
    }
    tracing::info!("Consumer loop finished");

    Ok(())
}
