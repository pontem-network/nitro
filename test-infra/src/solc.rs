use anyhow::Result;

async fn version_solc() -> Result<String> {
    todo!()
}

/// Version solc "0.8.19" is expected
///
/// OPCODE push0 is not yet supported, but will soon be available.
/// This means that solidity version 0.8.20 or higher can only be used with an evm-version lower than
/// the default shanghai (see instructions here to change that parameter in solc, or here to set the
/// solidity or evmVersion configuration parameters in hardhat). Versions up to 0.8.19 (included) are
/// fully compatible.
/// Source: https://developer.arbitrum.io/solidity-support
///
/// Error: Error checking the "solc" version or unsuitable "solc" version
///
pub async fn check_solc() -> Result<()> {
    todo!()
}
