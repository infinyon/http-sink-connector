mod config;
mod sink;

use anyhow::Result;
use config::HttpConfig;
use futures::SinkExt;
use sink::HttpSink;

use fluvio_connector_common::{connector, consumer::ConsumerStream, tracing::debug, Sink};

#[connector(sink)]
async fn start(config: HttpConfig, mut stream: impl ConsumerStream) -> Result<()> {
    debug!(?config);

    let sink = HttpSink::new(&config)?;
    let mut sink = sink.connect(None).await?;

    while let Some(item) = stream.next().await {
        let str = String::from_utf8(item?.as_ref().to_vec())?;
        sink.send(str).await?;
    }

    Ok(())
}
