// @todo
// Important ports
// RPC: 8547
// Sequencer Feed: 9642
// WebSocket: 8548
// WS port 8548 needs extra args to be opened. Please use these flags:
//      --ws.port=8548
//      --ws.addr=0.0.0.0
//      --ws.origins=*
//
// 6379 ?
// 8047 ?
// 8048 ?
// 8147 ?
// 8148 ?
// 8545 geth HTTP-RPC server listening port
// 8546 ?
// 8547 geth graphql
// 8548 - WebSocket
// 8551 geth authenticated execution API
// 8949 ?
// 9642 - Sequencer Feed
// 30303
// - 5052 Beacon Node API
// - 5054 metrics
// - 6174 web3signerEndpoint
// - 18550 mev-boost

use anyhow::Result;
use mimicaw::{Args, Test};
use tokio::task;
use web3::transports::Http;
use web3::Web3;

use test_infra::check_service::wait_up;
use test_infra::solc::check_solc;

mod eth;
mod mimicaw_helper;

use crate::eth::{new_account, root_account, unlock};
use crate::mimicaw_helper::TestHandleResultToOutcom;

const L1_ADDRESS: &str = "0x3f1Eae7D46d88F08fc2F8ed27FCb2AB183EB2d0E";

type TestFn = fn() -> task::JoinHandle<Result<()>>;

#[inline]
fn eth_http() -> String {
    "http://127.0.0.1:8545".to_string()
}

/// RUN NITRO: $ cd nitro-testnode; ./test-node.bash --dev
#[tokio::main]
async fn main() -> Result<()> {
    env_logger::builder().is_test(true).try_init()?;

    // pre
    assert!(wait_up(&eth_http()).await);
    check_solc().await?;

    // tests
    let args = Args::from_env().unwrap_or_else(|st| st.exit());

    let tests: Vec<Test<TestFn>> =
        vec![Test::<TestFn>::test("node_eth::create_new_account", || {
            task::spawn(async { create_new_account().await })
        })];

    mimicaw::run_tests(&args, tests, |_, test_fn: TestFn| {
        let handle = test_fn();
        async move { handle.await.to_outcome() }
    })
    .await
    .exit();
}

async fn create_new_account() -> Result<()> {
    let client = Web3::new(Http::new(&eth_http())?);

    let root_account_address = root_account(&client).await?;
    new_account(&client, root_account_address).await?;

    Ok(())
}
