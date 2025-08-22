// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "forge-std/Test.sol";
import "../src/VerifierSignaturePaymaster.sol";
import "../lib/account-abstraction/contracts/core/EntryPoint.sol";

contract PaymasterTest is Test {
    VerifierSignaturePaymaster paymaster;
    EntryPoint entryPoint;
    address verifier;
    uint256 verifierKey;
    
    function setUp() public {
        (verifier, verifierKey) = makeAddrAndKey("verifier");
        entryPoint = new EntryPoint();
        paymaster = new VerifierSignaturePaymaster(entryPoint, verifier);
        
        vm.deal(address(paymaster), 10 ether);
        paymaster.deposit{value: 10 ether}();
    }
    
    function testValidatePaymasterUserOp() public {
        // Create test UserOperation
        PackedUserOperation memory userOp = createTestUserOp();
        
        // We need to create the signature for the final hash, but we have a chicken-and-egg problem
        // Solution: Create signature with a placeholder, then recreate with the correct hash
        
        uint64 validUntil = uint64(block.timestamp + 3600);
        uint256 maxCost = 1000000;
        
        // First, create a temporary signature to get the final hash
        bytes memory tempSignature = new bytes(65); // Empty 65-byte signature
        bytes memory tempPaymasterData = abi.encodePacked(tempSignature, validUntil);
        userOp.paymasterAndData = tempPaymasterData;
        
        // Get the hash that will be used for validation
        bytes32 finalHash = entryPoint.getUserOpHash(userOp);
        
        // Now create the real signature using the final hash
        bytes memory realSignature = createVerifierSignature(
            finalHash,
            validUntil,
            maxCost
        );
        
        // Update paymaster data with real signature
        bytes memory realPaymasterData = abi.encodePacked(realSignature, validUntil);
        userOp.paymasterAndData = realPaymasterData;
        
        // Test validation
        vm.prank(address(entryPoint));
        (bytes memory context, uint256 validationData) = paymaster.validatePaymasterUserOp(
            userOp,
            finalHash,
            maxCost
        );
        
        assertEq(validationData, 0); // Success
    }
    
    function createTestUserOp() internal pure returns (PackedUserOperation memory) {
        return PackedUserOperation({
            sender: address(0x1234567890123456789012345678901234567890),
            nonce: 0,
            initCode: "",
            callData: "",
            accountGasLimits: bytes32(abi.encodePacked(uint128(100000), uint128(100000))),
            preVerificationGas: 21000,
            gasFees: bytes32(abi.encodePacked(uint128(20000000000), uint128(2000000000))),
            paymasterAndData: "",
            signature: ""
        });
    }
    
    function createVerifierSignature(bytes32 userOpHash, uint64 validUntil, uint256 maxCost) 
        internal view returns (bytes memory) {
        // Create message hash (same as in paymaster contract)
        bytes32 messageHash = keccak256(abi.encodePacked(
            userOpHash,
            validUntil,
            maxCost
        ));
        
        // Apply EIP-191 prefix (same as MessageHashUtils.toEthSignedMessageHash)
        bytes32 ethSignedHash = keccak256(abi.encodePacked(
            "\x19Ethereum Signed Message:\n32",
            messageHash
        ));
        
        // Sign with verifier key
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(verifierKey, ethSignedHash);
        
        // Return signature (65 bytes: r, s, v)
        return abi.encodePacked(r, s, v);
    }
}
