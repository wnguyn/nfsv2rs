mod rpc;
mod handle;
mod transport;
mod fs;

use std::path::PathBuf;


const ROOT: &'static str = "/home/will/mnt";

// janky; use ENV variables ater
pub struct Config {
    raw_url: String,
    conf: PathBuf,
}

impl Config {
    pub fn new(args: &str) -> Self {
        let path = PathBuf::from(args);
        Self {
            raw_url: format!("{args}"),
            conf: path,
        }
    }
}

impl Clone for Config {
    fn clone(&self) -> Self {
        Self {
            raw_url: self.raw_url.clone(),
            conf: self.conf.clone(),
        }
    }
}
#[tokio::main]
pub async fn main() -> anyhow::Result<()> {

    let handler = handle::make_handler("/export")?;
    tracing::info!("NFSv2 server starting, export root: /export");
    let cfg_ptr = Box::new(Config::new(ROOT));

    transport::serve(cfg_ptr, handler, "0.0.0.0:2049").await
}
