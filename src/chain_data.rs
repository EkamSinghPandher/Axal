use alloy::{primitives::U256, providers::{Provider, ProviderBuilder}, sol};
use eyre::Result;
use serde::{Deserialize, Serialize};

use crate::utils::convert_float_to_large_u64_9_decimals;

// Codegen from ABI file to interact with the contract.
sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    UNISWAPV3,
    "src/contract-abi/uniswapv3.json"
);

// ERC20 interface for getting token details
sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    ERC20,
    "src/contract-abi/erc20.json"
);

// Chain configuration struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    pub token0_name: String, 
    pub token1_name: String, 
    pub name: String,
    pub rpc_url: String,
    pub pool_addr: String,
}

// Chain configuration struct for 2 chains to compare the asset prices
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainComparisonConfig{
    pub chain_cfg_1: ChainConfig,
    pub chain_cfg_2: ChainConfig, 
    pub diff_threshold: u64
}

// Structure to hold proving inputs for price data for proving threashold 
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PriceDataProvingInputs {
    pub price_proving_pis_1: SingleChainProvingInputs,
    pub price_proving_pis_2: SingleChainProvingInputs,
    pub diff_threshold: u64
}

// Structure to hold proving inputs for price data for a singular chain
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SingleChainProvingInputs{
    pub block_number: u64,
    pub sqrt_price_x96: u64,
}

/// This is a method to get the proving inputs of the pool price data, this is mainly the sqrtx96price and block number. These will be used in the proof system.
pub async fn get_individual_chain_price_proving_inputs(chain: &ChainConfig)-> Result<SingleChainProvingInputs>{
     // Connect to the chain
     let provider = ProviderBuilder::new()
     .on_http(chain.rpc_url.parse()?);
 
    // Connect to the pool
    // Parse the address string into Alloy's Address type
    let pool_address = chain.pool_addr.parse()?;
    let pool = UNISWAPV3::new(pool_address, provider.clone());
    
    // Get current block number
    let block_number = provider.get_block_number().await?;
    
    // Get current price
    let slot_0_return = pool.slot0().call().await?;

    let sqrt_price_x96 = slot_0_return.sqrtPriceX96;
    let sqrt_price_x96_u256 = U256::from(sqrt_price_x96);


    // Convert to bytes first since there's no direct as_u128 method
    let sqrt_price_bytes = sqrt_price_x96_u256.to_be_bytes::<32>();
    
    // Take the lower 16 bytes for u128
    let mut sqrt_price_u128_bytes = [0u8; 16];
    sqrt_price_u128_bytes.copy_from_slice(&sqrt_price_bytes[16..32]);
    let sqrt_price_u128 = u128::from_be_bytes(sqrt_price_u128_bytes);

    let sqrt_price = sqrt_price_u128 as f64 / (1u128 << 96) as f64;
    let sqrt_price_int = convert_float_to_large_u64_9_decimals(sqrt_price);

    Ok(SingleChainProvingInputs {
       sqrt_price_x96: sqrt_price_int,
       block_number: block_number
    })
}


/// Generates the proving input used as the trace for the plonky2 proof. This trace includes price data from on chain as well as a block number so that 
/// others may verify that the price at the time of the block is correct.
pub async fn generate_proving_inputs(cfg: ChainComparisonConfig)-> Result<PriceDataProvingInputs>{
    let indiv_chain_data_1 = get_individual_chain_price_proving_inputs(&cfg.chain_cfg_1).await?;
    let indiv_chain_data_2 = get_individual_chain_price_proving_inputs(&cfg.chain_cfg_2).await?;

    return Ok(PriceDataProvingInputs{
        price_proving_pis_1: indiv_chain_data_1,
        price_proving_pis_2: indiv_chain_data_2,
        diff_threshold: cfg.diff_threshold
    })
}