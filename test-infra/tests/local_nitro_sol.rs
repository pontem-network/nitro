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
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Duration;
use tokio::task;
use web3::contract::Options;
use web3::transports::Http;
use web3::Web3;

use test_infra::check_service::wait_up;
use test_infra::solc::check_solc;

mod eth;
mod mimicaw_helper;
mod sol;
mod tmpdir;

use crate::eth::{new_account, root_account, unlock};
use crate::mimicaw_helper::TestHandleResultToOutcom;
use crate::sol::{build_sol, SolContract};
use crate::tmpdir::TmpDir;

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

    let tests: Vec<Test<TestFn>> = vec![
        Test::<TestFn>::test("node_eth::create_new_account", || {
            task::spawn(async { create_new_account().await })
        }),
        Test::<TestFn>::test("node_eth::deploy_contract", || {
            task::spawn(async { deploy_contract().await })
        }),
    ];

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

async fn deploy_contract() -> Result<()> {
    let contract_sol = SolContract::try_from_path("./tests/sol_sources/const_fn.sol").await?;

    let client = Web3::new(Http::new(&eth_http())?);

    let root_account_address = root_account(&client).await?;
    let (alice_address, alice_key) = new_account(&client, root_account_address).await?;

    let web3_contract =
        web3::contract::Contract::deploy(client.eth(), contract_sol.abi_str().as_bytes())?
            .confirmations(1)
            .poll_interval(Duration::from_secs(1))
            .options(Options::with(|opt| opt.gas = Some(3_000_000.into())))
            .execute(contract_sol.bin_hex(), (), alice_address)
            .await?;
    let contract_address = web3_contract.address();
    println!("Deployed at: 0x{contract_address:x}");

    //
    let result: u64 = web3_contract
        .query("const_fn_10", (), None, Options::default(), None)
        .await?;
    assert_eq!(result, 10);

    //
    let contract =
        web3::contract::Contract::new(client.eth(), contract_address, contract_sol.abi()?);
    let result: bool = web3_contract
        .query("const_fn_true", (), None, Options::default(), None)
        .await?;
    assert!(result);

    Ok(())
}
