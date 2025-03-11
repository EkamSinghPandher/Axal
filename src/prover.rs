use eyre::Result;
use plonky2::{fri::{reduction_strategies::FriReductionStrategy, FriConfig}, iop::{target::Target, witness::{PartialWitness, WitnessWrite}}, plonk::{circuit_builder::CircuitBuilder, circuit_data::CircuitConfig, config::{GenericConfig, PoseidonGoldilocksConfig}, proof::ProofWithPublicInputs, prover::prove}, util::timing::TimingTree};
use plonky2_field::types::Field;

use crate::{chain_data::{generate_proving_inputs, ChainComparisonConfig, PriceDataProvingInputs}, comparison::compare_gate_unsafe, utils::convert_float_to_large_u64_16_decimals};


pub const D: usize = 2;
pub type C = PoseidonGoldilocksConfig;
pub type F = <C as GenericConfig<D>>::F;
pub const MAX_POSITIVE_AMOUNT_LOG: usize = 62;
pub const MAX_POSITIVE_AMOUNT: u64 = (1 << MAX_POSITIVE_AMOUNT_LOG) - 1;
pub const P: u64 = 0xFFFFFFFF00000001;

pub const STANDARD_CONFIG: CircuitConfig = CircuitConfig {
    num_wires: 170, // plonky2-u32 U32RangeCheckGate requires 170 wires
    num_routed_wires: 80,
    num_constants: 2,
    use_base_arithmetic_gate: true,
    security_bits: 100,
    num_challenges: 2,
    zero_knowledge: false,
    max_quotient_degree_factor: 8,
    fri_config: FriConfig {
        rate_bits: 3,
        cap_height: 4,
        proof_of_work_bits: 16,
        reduction_strategy: FriReductionStrategy::ConstantArityBits(4, 5),
        num_query_rounds: 28,
    },
};

pub struct PriceCircuitTargets{
    pub pool_1_sqrt_price_x96_target: Target,
    pub pool_2_sqrt_price_x96_target: Target,
    pub pool_1_block_number_target: Target,
    pub pool_2_block_number_target: Target,
    pub diff_threshold: Target
}

impl PriceCircuitTargets{
    pub fn create_price_diff_circuit(
        builder: &mut CircuitBuilder<F,D>
    )->Self{
        let pool_1_sqrt_price_x96_target = builder.add_virtual_target();
        let pool_2_sqrt_price_x96_target = builder.add_virtual_target();
        let pool_1_block_number_target = builder.add_virtual_target();
        let pool_2_block_number_target = builder.add_virtual_target();
        let diff_threshold = builder.add_virtual_target();

        let price_1_target = builder.mul(pool_1_sqrt_price_x96_target, pool_1_sqrt_price_x96_target);
        let price_2_target = builder.mul(pool_2_sqrt_price_x96_target, pool_2_sqrt_price_x96_target);



        let is_first_price_greater_than_second = compare_gate_unsafe(builder, price_1_target, price_2_target);
        

        let price_diff_1_then_2_target = builder.sub(price_1_target, price_2_target);
        let price_diff_2_then_1_target = builder.sub(price_2_target, price_1_target);

        let price_diff_target = builder.select(is_first_price_greater_than_second, price_diff_1_then_2_target, price_diff_2_then_1_target);
        let is_price_diff_more_than_threshold = compare_gate_unsafe(builder, price_diff_target, diff_threshold);

        builder.register_public_input(pool_1_block_number_target);
        builder.register_public_input(pool_2_block_number_target);
        builder.register_public_input(diff_threshold);
        builder.register_public_input(is_price_diff_more_than_threshold.target);


        PriceCircuitTargets{
            pool_1_sqrt_price_x96_target,
            pool_2_sqrt_price_x96_target,
            pool_1_block_number_target,
            pool_2_block_number_target,
            diff_threshold
        }
    }

    pub fn set_price_diff_circuit(
        &self,
        pw: &mut PartialWitness<F>,
        proving_inputs: &PriceDataProvingInputs,
    ){
        
        pw.set_target(self.pool_1_sqrt_price_x96_target, F::from_canonical_u64(proving_inputs.price_proving_pis_1.sqrt_price_x96));
        pw.set_target(self.pool_2_sqrt_price_x96_target, F::from_canonical_u64(proving_inputs.price_proving_pis_2.sqrt_price_x96));
        pw.set_target(self.pool_1_block_number_target, F::from_canonical_u64(proving_inputs.price_proving_pis_1.block_number));
        pw.set_target(self.pool_2_block_number_target, F::from_canonical_u64(proving_inputs.price_proving_pis_2.block_number));
        pw.set_target(self.diff_threshold, F::from_canonical_u64(convert_float_to_large_u64_16_decimals(proving_inputs.diff_threshold)));

    }
}


pub struct Prover{
    pub builder: CircuitBuilder<F,D>,
    pub pw: PartialWitness<F>
}

impl Prover{
    pub fn new(config: CircuitConfig)->Self{
        Prover{
            builder: CircuitBuilder::new(config),
            pw: PartialWitness::new()
        }
    }

    pub async fn prove(&mut self, chain_cfg: ChainComparisonConfig)-> Result<ProofWithPublicInputs<F,C,D>>{
        println!("Generating proving inputs.");
        let proving_inputs = generate_proving_inputs(chain_cfg).await?;
        println!("Proving inputs generated: {:?}", proving_inputs);
        println!("Building circuit");
        let price_targets = PriceCircuitTargets::create_price_diff_circuit(&mut self.builder);
        price_targets.set_price_diff_circuit(&mut self.pw, &proving_inputs);

        let mut timing = TimingTree::new("transaction_proof", log::Level::Debug);
        let data = self.builder.clone().build::<C>();

        println!("Circuit built");
        println!("Initiating proof");
        let proof = prove(&data.prover_only, &data.common, self.pw.clone(), &mut timing).expect("Prove fail");
        println!("Proof complete");
        return Ok(proof);
    }
}
