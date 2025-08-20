// Simplified UserOperation validation utilities
use alloy::primitives::{Address, U256};
use crate::error::{AAError, Result};

/// Basic validation for addresses and common parameters
pub fn validate_address(address: Address, field_name: &str) -> Result<()> {
    if address == Address::ZERO {
        return Err(AAError::ValidationError(format!("{} cannot be zero address", field_name)));
    }
    Ok(())
}

/// Validate gas limits are within reasonable bounds
pub fn validate_gas_amount(gas: U256, field_name: &str, min: u64, max: u64) -> Result<()> {
    if gas > U256::from(max) {
        return Err(AAError::ValidationError(
            format!("{} too high: {} > {}", field_name, gas, max)
        ));
    }
    
    if gas < U256::from(min) {
        return Err(AAError::ValidationError(
            format!("{} too low: {} < {}", field_name, gas, min)
        ));
    }
    
    Ok(())
}

/// Validate gas fees are reasonable
pub fn validate_gas_fees(max_fee: U256, priority_fee: U256) -> Result<()> {
    const MAX_FEE_PER_GAS: u64 = 1000_000_000_000; // 1000 gwei
    const MAX_PRIORITY_FEE: u64 = 100_000_000_000; // 100 gwei
    const MIN_GAS_PRICE: u64 = 1_000_000_000; // 1 gwei
    
    validate_gas_amount(max_fee, "Max fee per gas", MIN_GAS_PRICE, MAX_FEE_PER_GAS)?;
    validate_gas_amount(priority_fee, "Max priority fee", 0, MAX_PRIORITY_FEE)?;
    
    if priority_fee > max_fee {
        return Err(AAError::ValidationError(
            "Max priority fee cannot be higher than max fee per gas".to_string()
        ));
    }
    
    Ok(())
}

/// Validate initCode format for account creation
pub fn validate_init_code(init_code: &[u8]) -> Result<()> {
    if init_code.is_empty() {
        return Ok(()); // Empty initCode is valid for existing accounts
    }

    // InitCode should be at least 20 bytes (factory address)
    if init_code.len() < 20 {
        return Err(AAError::ValidationError(
            "InitCode too short - must be at least 20 bytes".to_string()
        ));
    }

    // Extract factory address (first 20 bytes)
    let factory_address = Address::from_slice(&init_code[0..20]);
    if factory_address == Address::ZERO {
        return Err(AAError::ValidationError(
            "Factory address in initCode cannot be zero".to_string()
        ));
    }

    Ok(())
}

/// Validate paymaster data if present
pub fn validate_paymaster_data(paymaster_data: &[u8]) -> Result<()> {
    if paymaster_data.is_empty() {
        return Ok(()); // No paymaster is valid
    }
    
    // Paymaster data should be at least 20 bytes (paymaster address)
    if paymaster_data.len() < 20 {
        return Err(AAError::ValidationError(
            "Paymaster data too short - must be at least 20 bytes".to_string()
        ));
    }

    // Extract paymaster address (first 20 bytes)
    let paymaster_address = Address::from_slice(&paymaster_data[0..20]);
    if paymaster_address == Address::ZERO {
        return Err(AAError::ValidationError(
            "Paymaster address cannot be zero".to_string()
        ));
    }

    // If paymaster data is longer than 20 bytes, it should include gas limits
    if paymaster_data.len() > 20 && paymaster_data.len() < 52 {
        return Err(AAError::ValidationError(
            "Incomplete paymaster data - expected verification and post-op gas limits".to_string()
        ));
    }

    Ok(())
}

/// Basic validation helper for UserOperation components
pub fn validate_user_operation_basic(
    sender: Address,
    call_data: &[u8],
    init_code: &[u8],
    signature: &[u8],
) -> Result<()> {
    // Check sender is not zero
    validate_address(sender, "Sender")?;

    // Check that either callData or initCode is provided
    if call_data.is_empty() && init_code.is_empty() {
        return Err(AAError::ValidationError(
            "Both callData and initCode cannot be empty - account creation or execution required".to_string()
        ));
    }

    // InitCode validation
    validate_init_code(init_code)?;
    
    // Signature validation (basic check)
    if signature.is_empty() {
        return Err(AAError::ValidationError(
            "Signature cannot be empty".to_string()
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_address() {
        let valid_addr = Address::from([1u8; 20]);
        assert!(validate_address(valid_addr, "test").is_ok());
        
        let zero_addr = Address::ZERO;
        let result = validate_address(zero_addr, "test");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("test cannot be zero"));
    }

    #[test]
    fn test_validate_gas_amount() {
        // Valid gas amount
        let gas = U256::from(100000u64);
        assert!(validate_gas_amount(gas, "test", 50000, 200000).is_ok());
        
        // Too high
        let high_gas = U256::from(300000u64);
        let result = validate_gas_amount(high_gas, "test", 50000, 200000);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too high"));
        
        // Too low
        let low_gas = U256::from(10000u64);
        let result = validate_gas_amount(low_gas, "test", 50000, 200000);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too low"));
    }

    #[test]
    fn test_validate_gas_fees() {
        // Valid fees
        let max_fee = U256::from(20_000_000_000u64); // 20 gwei
        let priority_fee = U256::from(2_000_000_000u64); // 2 gwei
        assert!(validate_gas_fees(max_fee, priority_fee).is_ok());
        
        // Priority fee higher than max fee
        let result = validate_gas_fees(U256::from(10_000_000_000u64), U256::from(20_000_000_000u64));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be higher"));
    }

    #[test]
    fn test_validate_init_code() {
        // Valid initCode with factory address
        let init_code = [1u8; 24].to_vec(); // 20 bytes factory + 4 bytes call data
        assert!(validate_init_code(&init_code).is_ok());

        // Empty initCode (valid)
        assert!(validate_init_code(&[]).is_ok());

        // Too short initCode
        let init_code = [1u8; 10].to_vec();
        let result = validate_init_code(&init_code);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too short"));

        // Zero factory address
        let mut init_code = [1u8; 24].to_vec();
        init_code[0..20].fill(0); // Set factory address to zero
        let result = validate_init_code(&init_code);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Factory address"));
    }

    #[test]
    fn test_validate_user_operation_basic() {
        let sender = Address::from([1u8; 20]);
        let call_data = [0x12, 0x34];
        let init_code = [];
        let signature = [0x56, 0x78];
        
        // Valid operation
        assert!(validate_user_operation_basic(sender, &call_data, &init_code, &signature).is_ok());
        
        // Zero sender
        let result = validate_user_operation_basic(Address::ZERO, &call_data, &init_code, &signature);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Sender cannot be zero"));
        
        // Empty call data and init code
        let result = validate_user_operation_basic(sender, &[], &[], &signature);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Both callData and initCode cannot be empty"));
        
        // Empty signature
        let result = validate_user_operation_basic(sender, &call_data, &init_code, &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Signature cannot be empty"));
    }
}