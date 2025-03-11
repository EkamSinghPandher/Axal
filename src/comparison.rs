use plonky2::{
    field::extension::Extendable,
    hash::hash_types::RichField,
    iop::target::{BoolTarget, Target},
    plonk::circuit_builder::CircuitBuilder,
};


/// Checks if a integer is poitive in the 2's complement system by checking the 2 MSB's to see if both are 0
/// (this is a indicator of positive numbers).
fn is_positive<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    msb_x_64: BoolTarget,
    msb_x_63: BoolTarget,
) -> BoolTarget {
    // !msb_x_64 & !msb_x_63
    let not_msb_x_64 = builder.not(msb_x_64);
    let not_msb_x_63 = builder.not(msb_x_63);
    builder.and(not_msb_x_64, not_msb_x_63)
}


/// Only applicable for x, y in range [0, 2^62-1] and [p-2^62, p-1] and with the same sign.
///
/// Only run this if you are sure the signs are the same.
///
/// x >= y: return true
///
/// x < y: return false
pub fn compare_gate_unsafe<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    x: Target,
    y: Target,
) -> BoolTarget {
    // Assume x and y have the same sign
    // Only check x-y >= 0
    let diff: Target = builder.sub(x, y);
    let diff_bits: Vec<BoolTarget> = builder.split_le(diff, 64);
    is_positive(builder, diff_bits[63], diff_bits[62])
}


#[cfg(test)]
mod tests {
    use crate::{circuit_utils::test_util::run_circuit_test, prover::{F, MAX_POSITIVE_AMOUNT, MAX_POSITIVE_AMOUNT_LOG, P}};

    use super::*;
    use plonky2::{
        field::types::Field,
        iop::witness::{PartialWitness, WitnessWrite},
    };

    // Smallest positive 0
    const SMALLEST_POSITIVE: u64 = 0;
    // Largest positive 2^62 - 1
    const LARGEST_POSITIVE: u64 = MAX_POSITIVE_AMOUNT;
    // Smallest negative number p-2^62 corresponding to -2^62
    const SMALLEST_NEGATIVE: u64 = P - (1 << MAX_POSITIVE_AMOUNT_LOG);
    // Largest negative number p-1 corresponding to -1
    const LARGEST_NEGATIVE: u64 = P - 1;

    #[test]
    fn test_is_positive() {
        run_circuit_test(|builder, pw| {
            setup_is_positive_gate(builder, pw, F::from_canonical_u64(SMALLEST_POSITIVE), true);

            setup_is_positive_gate(builder, pw, F::from_canonical_u64(LARGEST_POSITIVE), true);

            setup_is_positive_gate(builder, pw, F::from_canonical_u64(LARGEST_NEGATIVE), false);

            setup_is_positive_gate(builder, pw, F::from_canonical_u64(SMALLEST_NEGATIVE), false);
        });
    }

    #[test]
    fn test_compare_gate_unsafe() {
        run_circuit_test(|builder, pw| {
            // Test largest and smallest positive numbers
            setup_compare_unsafe_gate(
                builder,
                pw,
                F::from_canonical_u64(LARGEST_POSITIVE),
                F::from_canonical_u64(SMALLEST_POSITIVE),
                true,
            );

            // Test largest and smallest positive numbers (reverse order)
            setup_compare_unsafe_gate(
                builder,
                pw,
                F::from_canonical_u64(SMALLEST_POSITIVE),
                F::from_canonical_u64(LARGEST_POSITIVE),
                false,
            );

            // x == y
            setup_compare_unsafe_gate(
                builder,
                pw,
                F::from_canonical_u64(12),
                F::from_canonical_u64(12),
                true,
            );

            // Test largest and smallest negative numbers
            setup_compare_unsafe_gate(
                builder,
                pw,
                F::from_canonical_u64(LARGEST_NEGATIVE),
                F::from_canonical_u64(SMALLEST_NEGATIVE),
                true,
            );

            // Test largest and smallest negative numbers
            setup_compare_unsafe_gate(
                builder,
                pw,
                F::from_canonical_u64(SMALLEST_NEGATIVE),
                F::from_canonical_u64(LARGEST_NEGATIVE),
                false,
            );

            // x == y (negative numbers)
            setup_compare_unsafe_gate(
                builder,
                pw,
                F::from_canonical_u64(16835058050987196417),
                F::from_canonical_u64(16835058050987196417),
                true,
            );
        });
    }


    /// Helper function for is_positive tests
    fn setup_is_positive_gate<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
        pw: &mut PartialWitness<F>,
        x: F,
        expected_result: bool,
    ) {
        let x_target = builder.add_virtual_target();
        let x_bits = builder.split_le(x_target, 64);
        let is_positive_target = is_positive(builder, x_bits[63], x_bits[62]);

        pw.set_target(x_target, x);
        pw.set_bool_target(is_positive_target, expected_result);
    }


    /// Helper function for unsafe compare tests
    fn setup_compare_unsafe_gate<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
        pw: &mut PartialWitness<F>,
        x: F,
        y: F,
        expected_result: bool,
    ) {
        let x_target = builder.add_virtual_target();
        let y_target = builder.add_virtual_target();
        let result_target = compare_gate_unsafe(builder, x_target, y_target);
        pw.set_target(x_target, x);
        pw.set_target(y_target, y);
        pw.set_bool_target(result_target, expected_result);
    }
}
