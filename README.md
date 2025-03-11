## Introduction
This codebase is written primarily in Rust, it requires the following nightly release: `rustc 1.81.0-nightly (fda509e81 2024-06-25)`
It can be run using the following CLI command: `cargo run -- --config <Path to chain config>`
For example, I have included
1. ETH-ARB USDC/USDT Pools: `cargo run -- --config chain-config.json`
2. ETH-BASE USDC/USDT Pools: `cargo run -- --config chain-config2.json`

Upon successful proving, you can see a `proof.json` file, this is the Plonky2 STARK proof of the statement that the difference in USDC prices on 
these 2 evm chains is greater than the diff_threshold defined in the config file. If this is true, then in you proof file, in the public inputs, 
you should see a portion known as public inputs that looks like this:

```json
  "public_inputs": [
    22021667, // Block number chain 1
    27441518, // Block number chain 2
    1, //Diff threshold (note it will be represented as the diff threshold you specified * 10^16)
    1 // Is statement true (this is true)
  ]
```

```json
  "public_inputs": [
    22021667, // Block number chain 1
    27441518, // Block number chain 2
    1, //Diff threshold (note it will be represented as the diff threshold you specified * 10^16)
    18446744069414584321 // Is statement true (this is false)
  ]
```

We will look mainly at the last value, if it is 1 then the statement is true and if it is 18446744069414584321, then the statement is false. 
If you're curious why 18446744069414584321, its because thats how we represent false, it is the largest value, known as the goldilocks prime and it
is the underlying prime field of this proof system. 

## Config Files
The config files are used to determine the 2 pools on different chains that the USDC price is compared to. Note, we need to ensure that the asset
we are comparing price to must be the same USDC-ETH : USDC-WETH, otherwise we cannot accuratly arbitrage the different assets.

The format of the config file is as such:


```json
{
    "chain_cfg_1": {
      "name": "Ethereum", //Chain name
      "token0_name": "USDC", // First asset should always be usdc
      "token1_name": "USDT", // Second asset can vary but should be same across the 2 chains 
      "rpc_url": "https://mainnet.infura.io/v3/{INFURA_PRIVATE_KEY}", // RPC url to fetch the price data for proving
      "pool_addr": "0x3416cF6C708Da44DB2624D63ea0AAef7113527C6" // Pool address of the pool
    },
      "chain_cfg_2": {
      "name": "Base",
      "token0_name": "USDC",
      "token1_name": "USDT",
      "rpc_url": "https://mainnet.base.org",
      "pool_addr": "0xd56da2b74ba826f19015e6b7dd9dae1903e85da1"
    },
    "diff_threshold": 1 //This is the difference threshold, it can be a floating point as well
}

```

## Code architecture
The code has 3 main components
1. Proving Input Genertor
2. Circuit Builder 
3. Witness Settor and proof generation


### Proving Input generator
In this stage, we parse the config files and we fetch the chain data from the chain. This is done in `chain_data.rs`. We fetch 2 main data points from the chain:
1. sqrtx96price : This is the sqrt(price) * 1 << 96. This is what we use to calculate price differences between pools and it is the price between the 2 pool assets

```rust
    // Get current price
    let slot_0_return = pool.slot0().call().await?;

    let sqrt_price_x96 = slot_0_return.sqrtPriceX96;
```

2. Block number: We Fetch the block number to embed into the proof. This way users can verify the proof to ensure that the inputs into the proof are actually correct (specifically price at a specific block). These are embedded into the proof in the public inputs and a fradulent proof with different public input values will not be 
verified

```rust
     // Get current block number
    let block_number = provider.get_block_number().await?;
```

### Circuit Builder and Witness Settor 
In this stage, we build the circuit wires. This creates the trace matrix for the circuit in Plonkish Arithimization and also creates the hash of the circuit data, 
so any fake circuits can be identified. We also provide the witness with the private and public inputs into the circuit, which populates the trace matrix. 

We do circuit building in this function
```rust
pub fn create_price_diff_circuit(
        builder: &mut CircuitBuilder<F,D>
    )->Self
```

We do witness setting in this function
```rust
pub fn set_price_diff_circuit(
        &self,
        pw: &mut PartialWitness<F>,
        proving_inputs: &PriceDataProvingInputs,
    )
```

###  Proof generation
In this stage, we run the prover to generate the FRI proof. Since the circuit is small it is relatively fast. After this we can verify this proof on the blockchain using a verifier contract. However this is not advised since FRI proofs are large and verification cost on chain is expensive. We can instead wrap this proof in the BN128 proof, followed by a wrap in a Groth16 proof with KZG. This makes proof size constant and reduces verification gas costs.