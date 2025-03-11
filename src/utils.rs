/**
 * This is useful for representing decimals as large ints. Makes our calculations easier in the circuit.
 */
pub fn convert_float_to_large_u64_9_decimals(float_value: f64) -> u64 {
    // Handle zero case
    if float_value == 0.0 {
        return 0;
    }
    
    // Handle invalid cases
    if !float_value.is_finite() || float_value < 0.0 {
        panic!("Input must be a finite positive number");
    }
    
    // Fixed decimal representation with 10^8 precision
    let large_int = (float_value * 1e8).round() as u64;
    
    return large_int;
}   

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_float_to_large_u64_conversion() {
        // Test 1: Simple integer
        let result = convert_float_to_large_u64_9_decimals(42.0);
        assert_eq!(result, 4200000000);

        // Test 2: Simple decimal
        let result = convert_float_to_large_u64_9_decimals(123.456);
        assert_eq!(result, 12345600000);

        // Test 3: Small number less than 1
        let result = convert_float_to_large_u64_9_decimals(0.00123);
        assert_eq!(result, 123000);

        // Test 4: Number with many decimal places
        let result = convert_float_to_large_u64_9_decimals(3.141592653589793);
        assert_eq!(result, 314159265);

        // Test 5: Zero
        let result = convert_float_to_large_u64_9_decimals(0.0);
        assert_eq!(result, 0);

        // Test 7: Number close to u64::MAX / 10^16 boundary
        // There is some error here, unfortunately not avoidable due to inaccuaracies in float -> fixed point representation
        let result = convert_float_to_large_u64_9_decimals(1844.6744073709);  // Close to u64::MAX / 10^16
        assert_eq!(result, 184467440737);

        // Test 8: Number just below 1
        let result = convert_float_to_large_u64_9_decimals(0.999999999);
        assert_eq!(result, 100000000);

        // Test 9: Precise financial value
        let result = convert_float_to_large_u64_9_decimals(156.73);
        assert_eq!(result, 15673000000);

        // Test 10: Very precise small number
        let result = convert_float_to_large_u64_9_decimals(0.0000123456789);
        assert_eq!(result, 1235);
    }

    #[test]
    #[should_panic(expected = "Input must be a finite positive number")]
    fn test_negative_input() {
        convert_float_to_large_u64_9_decimals(-1.0);
    }

    #[test]
    #[should_panic(expected = "Input must be a finite positive number")]
    fn test_infinity_input() {
        convert_float_to_large_u64_9_decimals(f64::INFINITY);
    }
}