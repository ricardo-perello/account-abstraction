// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";
import "@openzeppelin/contracts/utils/cryptography/MessageHashUtils.sol";
import "../lib/account-abstraction/contracts/core/BasePaymaster.sol";

contract VerifierSignaturePaymaster is BasePaymaster {
    using ECDSA for bytes32;
    using MessageHashUtils for bytes32;
    
    // Verifier address that authorizes gas sponsorship
    address public immutable verifier;
    
    // Events for monitoring
    event GasSponsored(address indexed user, uint256 gasCost, bytes32 userOpHash);
    
    constructor(IEntryPoint _entryPoint, address _verifier) BasePaymaster(_entryPoint) {
        verifier = _verifier;
    }
    
    function _validatePaymasterUserOp(
        PackedUserOperation calldata userOp,
        bytes32 userOpHash,
        uint256 maxCost
    ) internal virtual override returns (bytes memory context, uint256 validationData) {
        
        // Decode paymaster data (signature + expiration)
        PaymasterData memory data = _decodePaymasterData(userOp.paymasterAndData);
        
        // Create message hash for signature verification
        bytes32 messageHash = keccak256(abi.encodePacked(
            userOpHash,           // Bind to specific operation
            data.validUntil,      // Time window
            maxCost               // Gas cost limit
        ));
        
        // Use EIP-191 for signature verification
        bytes32 ethSignedHash = messageHash.toEthSignedMessageHash();
        address recoveredSigner = ethSignedHash.recover(data.signature);
        
        // Verify signature is from authorized verifier
        require(recoveredSigner == verifier, "Invalid verifier signature");
        require(block.timestamp <= data.validUntil, "Signature expired");
        
        // Log gas sponsorship
        emit GasSponsored(userOp.sender, maxCost, userOpHash);
        
        // Return success (empty context, 0 validation data)
        return ("", 0);
    }
    
    // Paymaster data structure
    struct PaymasterData {
        bytes signature;         // ECDSA signature from verifier
        uint64 validUntil;       // Expiration timestamp
    }
    
    // Decode paymaster data from UserOperation
    function _decodePaymasterData(bytes calldata paymasterAndData) 
        internal pure returns (PaymasterData memory data) {
        require(paymasterAndData.length >= 73, "Invalid paymaster data length");
        
        // Extract signature (65 bytes) and validUntil (8 bytes)
        data.signature = paymasterAndData[:65];
        data.validUntil = uint64(bytes8(paymasterAndData[65:73]));
    }
}
