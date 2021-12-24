use std::error::Error;

use tokio::runtime::Builder;

use vault::start_server;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(start_server("127.0.0.1:8888", "/data/vault_data", |_| {}))?;
    Ok(())
}
