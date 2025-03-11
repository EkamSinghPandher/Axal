#[cfg(test)]
pub mod test_util {
    use log::Level;
    use plonky2::{
        field::extension::Extendable,
        hash::hash_types::RichField,
        iop::witness::PartialWitness,
        plonk::{
            circuit_builder::CircuitBuilder, circuit_data::CircuitData, config::GenericConfig,
            prover::prove,
        },
        util::timing::TimingTree,
    };
    
    use core::panic;

    use crate::prover::{C, STANDARD_CONFIG};

    fn init_logger() {
        let _ =
            env_logger::builder().filter_level(log::LevelFilter::Debug).is_test(true).try_init();
    }
    /// Test runner
    pub fn run_circuit_test<T, F, const D: usize>(test: T) -> ()
    where
        T: FnOnce(&mut CircuitBuilder<F, D>, &mut PartialWitness<F>) -> () + panic::UnwindSafe,
        F: RichField + Extendable<D>,
        C: GenericConfig<D, F = F>,
    {
        init_logger();
        let mut builder = CircuitBuilder::<F, D>::new(STANDARD_CONFIG.clone());
        let mut pw: PartialWitness<F> = PartialWitness::<F>::new();
        test(&mut builder, &mut pw);
        // builder.print_gate_counts(0);
        let mut timing = TimingTree::new("prove", Level::Debug);
        let data = builder.build::<C>();
        let CircuitData { prover_only, common, verifier_only: _ } = &data;
        let proof = prove(&prover_only, &common, pw, &mut timing).expect("Prove fail");
        timing.print();
        data.verify(proof).expect("Verify fail")
    }
}
