use anyhow::{anyhow, ensure, Result};

use test_infra::check_service::wait_up;

const DEFAULT_REST_IP: &str = "127.0.0.1";
const DEFAULT_REST_PORT: u16 = 8551;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::builder().is_test(true).try_init()?;

    assert!(wait_up(&format!("http://{DEFAULT_REST_IP}:{DEFAULT_REST_PORT}")).await);

    Ok(())
}
