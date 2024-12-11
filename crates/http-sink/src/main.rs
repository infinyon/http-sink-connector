mod config;
mod sink;

use adaptive_backoff::prelude::{
    Backoff, BackoffBuilder, ExponentialBackoff, ExponentialBackoffBuilder,
};
use anyhow::{anyhow, Result};
use config::HttpConfig;
use fluvio::{
    consumer::Record,
    dataplane::{bytes::Bytes, link::ErrorCode},
};
use futures::{SinkExt, StreamExt};
use sink::HttpSink;

use fluvio_connector_common::{
    connector,
    consumer::ConsumerStream,
    tracing::{self, debug, error, info, warn},
    LocalBoxSink, Sink,
};

const SIGNATURES: &str = concat!("InfinyOn HTTP Sink Connector ", env!("CARGO_PKG_VERSION"));

#[connector(sink)]
async fn start(config: HttpConfig, mut stream: impl ConsumerStream) -> Result<()> {
    let mut backoff = backoff_init(&config)?;
    debug!(?config);

    let sink = HttpSink::new(&config)?;
    let mut sink = sink.connect(None).await?;

    info!("Starting {SIGNATURES}");
    while let Some(item) = stream.next().await {
        tracing::debug!("Received record in consumer");
        if let Err(err) = process_item(&mut sink, &mut backoff, &config, item).await {
            error!("Error processing item: {}", err);
        }
    }
    info!("Consumer loop finished");

    Ok(())
}

async fn process_item(
    sink: &mut LocalBoxSink<Bytes>,
    backoff: &mut ExponentialBackoff,
    config: &HttpConfig,
    item: Result<Record, ErrorCode>,
) -> Result<()> {
    let item: Bytes = item?.into_inner().into_value().into_vec().into();
    loop {
        match sink.send(item.clone()).await {
            Ok(_) => {
                backoff.reset();
                break;
            }
            Err(err) => {
                error!("Error sending operation to sink: {}", err);
                *sink = HttpSink::new(config)?.connect(None).await?;
                backoff_and_wait(backoff, config).await?;
            }
        }
    }

    Ok(())
}

async fn backoff_and_wait(backoff: &mut ExponentialBackoff, config: &HttpConfig) -> Result<()> {
    let wait = backoff.wait();
    if wait < config.backoff_max {
        warn!(
            "Waiting {} before next attempting to db",
            humantime::format_duration(wait)
        );
        fluvio_future::timer::sleep(wait).await;
        Ok(())
    } else {
        let err_msg = "Max retry on Http request, shutting down";
        error!(err_msg);
        Err(anyhow!(err_msg))
    }
}

fn backoff_init(config: &HttpConfig) -> Result<ExponentialBackoff> {
    ExponentialBackoffBuilder::default()
        .factor(1.5)
        .min(config.backoff_min)
        .max(config.backoff_max)
        .build()
}
