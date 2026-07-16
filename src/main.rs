mod rpc;
mod handle;
mod transport;

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {

    let handler = handle::make_handler("/export")?;
    tracing::info!("NFSv2 server starting, export root: /export");

    transport::serve(handler, "0.0.0.0:2049").await
}
