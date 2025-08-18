// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title UserOperation
 * @dev ERC-4337 UserOperation struct
 */
struct UserOperation {
    address sender; // The account making the operation
    uint256 nonce; // Anti-replay parameter
    bytes initCode; // The initCode for account creation (if any)
    bytes callData; // The data to pass to the sender during execution
    uint256 callGasLimit; // The amount of gas to allocate the main execution call
    uint256 verificationGasLimit; // The amount of gas to allocate for the verification step
    uint256 preVerificationGas; // The amount of gas to pay for to compensate the bundler for pre-verification execution and calldata
    uint256 maxFeePerGas; // Maximum fee per gas (similar to EIP-1559 gasPrice)
    uint256 maxPriorityFeePerGas; // Maximum priority fee per gas (similar to EIP-1559 maxPriorityFeePerGas)
    bytes paymasterAndData; // Contract address + data for the paymaster
    bytes signature; // Data passed into the account along with the nonce during the verification step
}
