// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "forge-std/Test.sol";
import "../src/AAAccount.sol";
import "@account-abstraction/contracts/core/EntryPoint.sol";

contract MockEntryPoint {
    // Mock EntryPoint that can call validateUserOp
}

contract AAAccountTest is Test {
    AAAccount public aaAccount;
    EntryPoint public entryPoint;
    
    address public owner1 = address(0x1);
    address public owner2 = address(0x2);
    address public nonOwner = address(0x3);
    address public targetAddress = address(0x123);
    
    // Private keys for testing
    uint256 public owner1PrivateKey = 0x1234567890123456789012345678901234567890123456789012345678901234;
    uint256 public owner2PrivateKey = 0x2345678901234567890123456789012345678901234567890123456789012345;
    
    // Test data
    bytes public testData = hex"1234";
    
    function setUp() public {
        // Derive actual addresses from private keys
        owner1 = vm.addr(owner1PrivateKey);
        owner2 = vm.addr(owner2PrivateKey);
        
        // Deploy EntryPoint
        entryPoint = new EntryPoint();
        
        // Deploy AAAccount
        aaAccount = new AAAccount(entryPoint, owner1);
    }
    
    function test_constructor_sets_initial_owner() public {
        assertTrue(aaAccount.isOwner(owner1));
        assertEq(aaAccount.ownerCount(), 1);
        
        address[] memory owners = aaAccount.getOwners();
        assertEq(owners.length, 1);
        assertEq(owners[0], owner1);
    }
    
    function test_addOwner_success() public {
        vm.prank(owner1);
        aaAccount.addOwner(owner2);
        
        assertTrue(aaAccount.isOwner(owner2));
        assertEq(aaAccount.ownerCount(), 2);
        
        address[] memory owners = aaAccount.getOwners();
        assertEq(owners.length, 2);
        assertTrue(contains(owners, owner1));
        assertTrue(contains(owners, owner2));
    }
    
    function test_addOwner_fails_if_not_owner() public {
        vm.prank(nonOwner);
        vm.expectRevert("AAAccount: caller is not an owner");
        aaAccount.addOwner(owner2);
    }
    
    function test_removeOwner_success() public {
        // Add second owner first
        vm.prank(owner1);
        aaAccount.addOwner(owner2);
        
        // Remove second owner
        vm.prank(owner1);
        aaAccount.removeOwner(owner2);
        
        assertFalse(aaAccount.isOwner(owner2));
        assertEq(aaAccount.ownerCount(), 1);
        assertTrue(aaAccount.isOwner(owner1));
    }
    
    function test_removeOwner_fails_if_removing_self() public {
        vm.prank(owner1);
        vm.expectRevert("AAAccount: cannot remove self");
        aaAccount.removeOwner(owner1);
    }
    
    function test_removeOwner_fails_if_removing_last_owner() public {
        // Add a second owner first
        vm.prank(owner1);
        aaAccount.addOwner(owner2);
        
        // Remove the second owner
        vm.prank(owner1);
        aaAccount.removeOwner(owner2);
        
        // Now try to remove the last owner (owner1)
        vm.prank(owner1);
        vm.expectRevert("AAAccount: cannot remove self");
        aaAccount.removeOwner(owner1);
    }
    
    function test_execute_by_owner() public {
        vm.prank(owner1);
        aaAccount.execute(targetAddress, 0, testData);
        // Should not revert
    }
    
    function test_execute_by_entrypoint() public {
        vm.prank(address(entryPoint));
        aaAccount.execute(targetAddress, 0, testData);
        // Should not revert
    }
    
    function test_execute_fails_if_not_owner_or_entrypoint() public {
        vm.prank(nonOwner);
        vm.expectRevert("account: not Owner or EntryPoint");
        aaAccount.execute(targetAddress, 0, testData);
    }
    
    function test_getNonce_returns_correct_value() public {
        uint256 nonceValue = aaAccount.getNonce(999);
        assertEq(nonceValue, 0, "Unused nonce should return 0");
    }
    
    function test_validateUserOp_success() public {
        // Create a valid UserOperation
        bytes32 userOpHash = keccak256("test hash");
        bytes memory signature = createValidSignature(userOpHash, owner1);
        
        // Create PackedUserOperation (simplified for testing)
        PackedUserOperation memory userOp = PackedUserOperation({
            sender: address(aaAccount),
            nonce: 100, // Use a different nonce to avoid conflicts
            initCode: "",
            callData: "",
            accountGasLimits: 0, // Packed gas limits (callGasLimit and verificationGasLimit)
            preVerificationGas: 100000,
            gasFees: 0, // Packed gas fees (maxFeePerGas and maxPriorityFeePerGas)
            paymasterAndData: "",
            signature: signature
        });
        
        // Call validateUserOp through the actual EntryPoint
        vm.prank(address(entryPoint));
        uint256 result = aaAccount.validateUserOp(userOp, userOpHash, 0);
        assertEq(result, 0, "Validation should return 0 for success");
        
        // Verify nonce was marked as used
        uint256 nonceValue = aaAccount.getNonce(100);
        assertEq(nonceValue, 1, "Nonce should be marked as used after validation");
    }
    
    function test_validateUserOp_fails_if_not_entrypoint() public {
        bytes32 userOpHash = keccak256("test hash");
        bytes memory signature = createValidSignature(userOpHash, owner1);
        
        PackedUserOperation memory userOp = PackedUserOperation({
            sender: address(aaAccount),
            nonce: 200, // Use a different nonce
            initCode: "",
            callData: "",
            accountGasLimits: 0, // Packed gas limits (callGasLimit and verificationGasLimit)
            preVerificationGas: 100000,
            gasFees: 0, // Packed gas fees (maxFeePerGas and maxPriorityFeePerGas)
            paymasterAndData: "",
            signature: signature
        });
        
        vm.prank(nonOwner);
        vm.expectRevert("account: not from EntryPoint");
        aaAccount.validateUserOp(userOp, userOpHash, 0);
    }
    
    // Helper function to create valid ECDSA signatures for testing
    function createValidSignature(bytes32 hash, address signer) internal view returns (bytes memory) {
        uint256 privateKey;
        if (signer == owner1) {
            privateKey = owner1PrivateKey;
        } else if (signer == owner2) {
            privateKey = owner2PrivateKey;
        } else {
            revert("Unknown signer");
        }
        
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(privateKey, hash);
        return abi.encodePacked(r, s, v);
    }
    
    // Helper function to check if an array contains a specific address
    function contains(address[] memory arr, address target) internal pure returns (bool) {
        for (uint256 i = 0; i < arr.length; i++) {
            if (arr[i] == target) {
                return true;
            }
        }
        return false;
    }
}
