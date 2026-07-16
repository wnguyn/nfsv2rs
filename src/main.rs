mod rpc;
mod nfs;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let _handler = nfs::NfsHandler::new("/export")?;

    Ok(())
}
