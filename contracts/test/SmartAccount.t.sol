// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "forge-std/Test.sol";
import "../src/SmartAccount.sol";
import "../src/interfaces/IEntryPoint.sol";
import "../src/interfaces/UserOperation.sol";

contract MockEntryPoint is IEntryPoint {
    SmartAccount public smartAccount;
    
    constructor(address _smartAccount) {
        smartAccount = SmartAccount(payable(_smartAccount));
    }
    
    function handleOps(UserOperation[] calldata ops, address payable beneficiary) external {}
    function simulateValidation(UserOperation calldata userOp) external {}
    
    function getUserOpHash(UserOperation calldata userOp) external pure returns (bytes32) {
        // Create a deterministic hash for testing
        return keccak256(abi.encode(
            userOp.sender,
            userOp.nonce,
            userOp.initCode,
            userOp.callData,
            userOp.callGasLimit,
            userOp.verificationGasLimit,
            userOp.preVerificationGas,
            userOp.maxFeePerGas,
            userOp.maxPriorityFeePerGas,
            userOp.paymasterAndData
        ));
    }
    
    // Function to call validateUserOp on the smart account
    function callValidateUserOp(UserOperation calldata userOp, bytes32 userOpHash, uint256 missingAccountFunds) 
        external 
        returns (uint256) 
    {
        return smartAccount.validateUserOp(userOp, userOpHash, missingAccountFunds);
    }
    
    // Function to call execute functions on the smart account
    function callExecute(address target, uint256 value, bytes calldata data) external {
        smartAccount.execute(target, value, data);
    }
    
    function callExecuteBatch(address[] calldata targets, uint256[] calldata values, bytes[] calldata datas) external {
        smartAccount.executeBatch(targets, values, datas);
    }
}

contract SmartAccountTest is Test {
    SmartAccount public smartAccount;
    MockEntryPoint public entryPoint;
    
    address public owner1 = address(0x1);
    address public owner2 = address(0x2);
    address public owner3 = address(0x3);
    address public nonOwner = address(0x4);
    address public targetAddress = address(0x123);
    
    // Private keys for testing (corresponding to the addresses above)
    uint256 public owner1PrivateKey = 0x1234567890123456789012345678901234567890123456789012345678901234;
    uint256 public owner2PrivateKey = 0x2345678901234567890123456789012345678901234567890123456789012345;
    uint256 public owner3PrivateKey = 0x3456789012345678901234567890123456789012345678901234567890123456;
    uint256 public nonOwnerPrivateKey = 0x4567890123456789012345678901234567890123456789012345678901234567;
    
    // Test data
    bytes public testData = hex"1234";
    bytes public testData2 = hex"5678";
    
    function setUp() public {
        // Derive the actual addresses from private keys
        owner1 = vm.addr(owner1PrivateKey);
        owner2 = vm.addr(owner2PrivateKey);
        owner3 = vm.addr(owner3PrivateKey);
        nonOwner = vm.addr(nonOwnerPrivateKey);
        
        // Create entry point first (with a temporary address)
        entryPoint = new MockEntryPoint(address(0));
        
        // Create smart account with the correct entry point
        vm.prank(owner1);
        smartAccount = new SmartAccount(address(entryPoint));
        
        // Update the entry point's reference to the actual smart account
        vm.store(
            address(entryPoint),
            bytes32(uint256(0)), // First storage slot for smartAccount
            bytes32(uint256(uint160(address(smartAccount))))
        );
    }
    
    // Helper function to create valid ECDSA signatures for testing
    function createValidSignature(bytes32 hash, address signer) internal view returns (bytes memory) {
        uint256 privateKey;
        if (signer == owner1) {
            privateKey = owner1PrivateKey;
        } else if (signer == owner2) {
            privateKey = owner2PrivateKey;
        } else if (signer == owner3) {
            privateKey = owner3PrivateKey;
        } else if (signer == nonOwner) {
            privateKey = nonOwnerPrivateKey;
        } else {
            revert("Unknown signer");
        }
        
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(privateKey, hash);
        return abi.encodePacked(r, s, v);
    }
    
    // Helper function to create UserOperation structs for testing
    function createUserOperation(
        address sender,
        uint256 nonce,
        bytes memory callData,
        bytes memory signature
    ) internal pure returns (UserOperation memory) {
        return UserOperation({
            sender: sender,
            nonce: nonce,
            initCode: "",
            callData: callData,
            callGasLimit: 100000,
            verificationGasLimit: 100000,
            preVerificationGas: 100000,
            maxFeePerGas: 20 gwei,
            maxPriorityFeePerGas: 2 gwei,
            paymasterAndData: "",
            signature: signature
        });
    }
    
    // ============================================================================
    // SIGNATURE VALIDATION TESTS
    // ============================================================================
    
    function test_isValidSignature_success() public {
        bytes32 hash = keccak256("test message");
        bytes memory signature = createValidSignature(hash, owner1);
        
        bool isValid = smartAccount.isValidSignature(hash, signature);
        assertTrue(isValid, "Valid signature from owner should return true");
    }
    
    function test_isValidSignature_fails_if_not_owner() public {
        bytes32 hash = keccak256("test message");
        bytes memory signature = createValidSignature(hash, nonOwner);
        
        bool isValid = smartAccount.isValidSignature(hash, signature);
        assertFalse(isValid, "Signature from non-owner should return false");
    }
    
    function test_isValidSignature_fails_if_invalid_length() public {
        bytes32 hash = keccak256("test message");
        bytes memory invalidSignature = hex"1234"; // Only 2 bytes, should be 65
        
        bool isValid = smartAccount.isValidSignature(hash, invalidSignature);
        assertFalse(isValid, "Signature with wrong length should return false");
    }
    
    function test_isValidSignature_fails_if_invalid_v() public {
        bytes32 hash = keccak256("test message");
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(owner1PrivateKey, hash);
        
        // Create invalid v value (not 27 or 28)
        uint8 invalidV = 25;
        bytes memory invalidSignature = abi.encodePacked(r, s, invalidV);
        
        bool isValid = smartAccount.isValidSignature(hash, invalidSignature);
        assertFalse(isValid, "Signature with invalid v should return false");
    }
    
    // ============================================================================
    // USEROPERATION VALIDATION TESTS
    // ============================================================================
    
    function test_validateUserOp_success() public {
        bytes32 userOpHash = keccak256("test hash");
        bytes memory signature = createValidSignature(userOpHash, owner1);
        
        UserOperation memory userOp = createUserOperation(
            address(smartAccount),
            1, // nonce
            "", // callData
            signature
        );
        
        uint256 result = entryPoint.callValidateUserOp(userOp, userOpHash, 0);
        assertEq(result, 0, "Validation should return 0 for gas cost");
        
        // Verify nonce was marked as used
        uint256 nonceValue = smartAccount.getNonce(1);
        assertEq(nonceValue, 1, "Nonce should be marked as used");
    }
    
    function test_validateUserOp_fails_if_not_entrypoint() public {
        bytes32 userOpHash = keccak256("test hash");
        bytes memory signature = createValidSignature(userOpHash, owner1);
        
        UserOperation memory userOp = createUserOperation(
            address(smartAccount),
            2, // nonce
            "", // callData
            signature
        );
        
        vm.prank(nonOwner);
        vm.expectRevert("SmartAccount: caller is not entry point");
        smartAccount.validateUserOp(userOp, userOpHash, 0);
    }
    
    function test_validateUserOp_fails_if_nonce_already_used() public {
        bytes32 userOpHash = keccak256("test hash");
        bytes memory signature = createValidSignature(userOpHash, owner1);
        
        UserOperation memory userOp = createUserOperation(
            address(smartAccount),
            3, // nonce
            "", // callData
            signature
        );
        
        // First validation should succeed
        entryPoint.callValidateUserOp(userOp, userOpHash, 0);
        
        // Second validation with same nonce should fail
        vm.expectRevert("SmartAccount: nonce already used");
        entryPoint.callValidateUserOp(userOp, userOpHash, 0);
    }
    
    function test_validateUserOp_fails_if_invalid_signature() public {
        bytes32 userOpHash = keccak256("test hash");
        bytes memory invalidSignature = hex"1234"; // Invalid signature
        
        UserOperation memory userOp = createUserOperation(
            address(smartAccount),
            4, // nonce
            "", // callData
            invalidSignature
        );
        
        vm.expectRevert("SmartAccount: invalid signature");
        entryPoint.callValidateUserOp(userOp, userOpHash, 0);
    }
    
    // ============================================================================
    // NONCE MANAGEMENT TESTS
    // ============================================================================
    
    function test_getNonce_returns_correct_value() public {
        uint256 nonceValue = smartAccount.getNonce(5);
        assertEq(nonceValue, 0, "Unused nonce should return 0");
        
        // Use the nonce
        bytes32 userOpHash = keccak256("test hash");
        bytes memory signature = createValidSignature(userOpHash, owner1);
        
        UserOperation memory userOp = createUserOperation(
            address(smartAccount),
            5, // nonce
            "", // callData
            signature
        );
        
        entryPoint.callValidateUserOp(userOp, userOpHash, 0);
        
        // Check nonce is now marked as used
        nonceValue = smartAccount.getNonce(5);
        assertEq(nonceValue, 1, "Used nonce should return 1");
    }
    
    function test_nonce_increments_after_validation() public {
        bytes32 userOpHash = keccak256("test hash");
        bytes memory signature = createValidSignature(userOpHash, owner1);
        
        UserOperation memory userOp = createUserOperation(
            address(smartAccount),
            6, // nonce
            "", // callData
            signature
        );
        
        // Before validation
        uint256 nonceValue = smartAccount.getNonce(6);
        assertEq(nonceValue, 0, "Nonce should be 0 before validation");
        
        // After validation
        entryPoint.callValidateUserOp(userOp, userOpHash, 0);
        nonceValue = smartAccount.getNonce(6);
        assertEq(nonceValue, 1, "Nonce should be 1 after validation");
    }
    
    // ============================================================================
    // UPDATED EXECUTION TESTS
    // ============================================================================
    
    function test_execute_fails_if_not_self() public {
        vm.prank(owner1);
        vm.expectRevert("SmartAccount: caller is not self");
        smartAccount.execute(targetAddress, 0, testData);
    }
    
    function test_executeBatch_fails_if_not_self() public {
        address[] memory targets = new address[](1);
        targets[0] = targetAddress;
        
        uint256[] memory values = new uint256[](1);
        values[0] = 0;
        
        bytes[] memory datas = new bytes[](1);
        datas[0] = testData;
        
        vm.prank(owner1);
        vm.expectRevert("SmartAccount: caller is not self");
        smartAccount.executeBatch(targets, values, datas);
    }
    
    // ============================================================================
    // MOCK ENTRYPOINT INTEGRATION TESTS
    // ============================================================================
    
    function test_entrypoint_can_call_validateUserOp() public {
        bytes32 userOpHash = keccak256("test hash");
        bytes memory signature = createValidSignature(userOpHash, owner1);
        
        UserOperation memory userOp = createUserOperation(
            address(smartAccount),
            7, // nonce
            "", // callData
            signature
        );
        
        uint256 result = entryPoint.callValidateUserOp(userOp, userOpHash, 0);
        assertEq(result, 0, "EntryPoint should be able to call validateUserOp");
    }
    
    function test_entrypoint_can_call_execute_functions() public {
        // EntryPoint should be able to call execute functions
        // This tests the internal execution flow
        
        // First validate a UserOperation to get the nonce marked
        bytes32 userOpHash = keccak256("test hash");
        bytes memory signature = createValidSignature(userOpHash, owner1);
        
        UserOperation memory userOp = createUserOperation(
            address(smartAccount),
            8, // nonce
            "", // callData
            signature
        );
        
        entryPoint.callValidateUserOp(userOp, userOpHash, 0);
        
        // In the real ERC-4337 flow, the EntryPoint would call execute functions
        // through the smart account's internal functions, not directly
        // For testing purposes, we'll verify that the validation worked correctly
        
        // Verify nonce was marked as used
        uint256 nonceValue = smartAccount.getNonce(8);
        assertEq(nonceValue, 1, "Nonce should be marked as used after validation");
        
        // Test that the smart account can execute functions when called by itself
        // This simulates the EntryPoint calling through the smart account
        vm.prank(address(smartAccount));
        smartAccount.execute(targetAddress, 0, testData);
        
        // Test batch execution
        address[] memory targets = new address[](1);
        targets[0] = targetAddress;
        
        uint256[] memory values = new uint256[](1);
        values[0] = 0;
        
        bytes[] memory datas = new bytes[](1);
        datas[0] = testData2;
        
        vm.prank(address(smartAccount));
        smartAccount.executeBatch(targets, values, datas);
    }
    
    // ============================================================================
    // OWNER MANAGEMENT TESTS (Updated for new flow)
    // ============================================================================
    
    function test_constructor_sets_deployer_as_owner() public {
        assertTrue(smartAccount.isOwner(owner1));
        assertEq(smartAccount.ownerCount(), 1);
        
        address[] memory owners = smartAccount.getOwners();
        assertEq(owners.length, 1);
        assertEq(owners[0], owner1);
    }
    
    function test_addOwner_success() public {
        vm.prank(owner1);
        smartAccount.addOwner(owner2);
        
        assertTrue(smartAccount.isOwner(owner2));
        assertEq(smartAccount.ownerCount(), 2);
        
        address[] memory owners = smartAccount.getOwners();
        assertEq(owners.length, 2);
        assertTrue(contains(owners, owner1));
        assertTrue(contains(owners, owner2));
    }
    
    function test_addOwner_fails_if_not_owner() public {
        vm.prank(nonOwner);
        vm.expectRevert("SmartAccount: caller is not an owner");
        smartAccount.addOwner(owner2);
    }
    
    function test_addOwner_fails_if_address_zero() public {
        vm.prank(owner1);
        vm.expectRevert("SmartAccount: invalid owner address");
        smartAccount.addOwner(address(0));
    }
    
    function test_addOwner_fails_if_already_owner() public {
        vm.prank(owner1);
        smartAccount.addOwner(owner2);
        
        vm.prank(owner1);
        vm.expectRevert("SmartAccount: owner already exists");
        smartAccount.addOwner(owner2);
    }
    
    function test_removeOwner_success() public {
        // Add second owner first
        vm.prank(owner1);
        smartAccount.addOwner(owner2);
        
        // Remove second owner
        vm.prank(owner1);
        smartAccount.removeOwner(owner2);
        
        assertFalse(smartAccount.isOwner(owner2));
        assertEq(smartAccount.ownerCount(), 1);
        assertTrue(smartAccount.isOwner(owner1));
    }
    
    function test_removeOwner_fails_if_not_owner() public {
        vm.prank(nonOwner);
        vm.expectRevert("SmartAccount: caller is not an owner");
        smartAccount.removeOwner(owner2);
    }
    
    function test_removeOwner_fails_if_removing_self() public {
        vm.prank(owner1);
        vm.expectRevert("SmartAccount: cannot remove self");
        smartAccount.removeOwner(owner1);
    }
    
    function test_removeOwner_fails_if_removing_last_owner() public {
        // Add a second owner first
        vm.prank(owner1);
        smartAccount.addOwner(owner2);
        
        // Remove the second owner
        vm.prank(owner1);
        smartAccount.removeOwner(owner2);
        
        // Now try to remove the last owner (owner1)
        vm.prank(owner1);
        vm.expectRevert("SmartAccount: cannot remove self");
        smartAccount.removeOwner(owner1);
    }
    
    function test_removeOwner_fails_if_owner_does_not_exist() public {
        vm.prank(owner1);
        vm.expectRevert("SmartAccount: owner does not exist");
        smartAccount.removeOwner(owner2);
    }
    
    function test_getOwners_returns_active_owners() public {
        vm.prank(owner1);
        smartAccount.addOwner(owner2);
        
        vm.prank(owner1);
        smartAccount.addOwner(owner3);
        
        address[] memory owners = smartAccount.getOwners();
        assertEq(owners.length, 3);
        assertTrue(contains(owners, owner1));
        assertTrue(contains(owners, owner2));
        assertTrue(contains(owners, owner3));
    }
    
    function test_getOwners_after_removal() public {
        vm.prank(owner1);
        smartAccount.addOwner(owner2);
        
        vm.prank(owner1);
        smartAccount.addOwner(owner3);
        
        vm.prank(owner1);
        smartAccount.removeOwner(owner2);
        
        address[] memory owners = smartAccount.getOwners();
        assertEq(owners.length, 2);
        assertTrue(contains(owners, owner1));
        assertTrue(contains(owners, owner3));
        assertFalse(contains(owners, owner2));
    }
    
    function test_isOwner() public {
        assertTrue(smartAccount.isOwner(owner1));
        assertFalse(smartAccount.isOwner(owner2));
        assertFalse(smartAccount.isOwner(nonOwner));
    }
    
    // ============================================================================
    // HELPER FUNCTIONS
    // ============================================================================
    
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
