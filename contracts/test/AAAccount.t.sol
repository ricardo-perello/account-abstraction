// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "forge-std/Test.sol";
import "../src/AAAccount.sol";
import "../src/AAAccountFactory.sol";
import "@account-abstraction/contracts/core/EntryPoint.sol";
import "@account-abstraction/contracts/core/UserOperationLib.sol";
import "@openzeppelin/contracts/utils/cryptography/MessageHashUtils.sol";


contract AAAccountTest is Test {
    using UserOperationLib for PackedUserOperation;
    
    EntryPoint public entryPoint;
    AAAccountFactory public factory;
    AAAccount public account;
    
    // Test accounts (using Anvil's pre-funded accounts)
    address public owner1 = address(0x70997970C51812dc3A010C7d01b50e0d17dc79C8);
    address public owner2 = address(0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC);
    address public owner3 = address(0x90F79bf6EB2c4f870365E785982E1f101E93b906);
    
    // Private keys for signing (Anvil's standard keys)
    uint256 public owner1Key = 0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d;
    uint256 public owner2Key = 0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a;
    
    // Receive function to accept ETH for bundler rewards
    receive() external payable {}
    
    function setUp() public {
        // Deploy EntryPoint
        entryPoint = new EntryPoint();
        
        // Deploy Factory
        factory = new AAAccountFactory(IEntryPoint(address(entryPoint)));
        
        // Fund test accounts
        vm.deal(owner1, 100 ether);
        vm.deal(owner2, 100 ether);
        vm.deal(owner3, 100 ether);
    }
    
    function testCreateSingleOwnerAccount() public {
        uint256 salt = uint256(keccak256("test-salt-1"));
        
        // Create account with single owner
        AAAccount newAccount = factory.createAccountDirect(owner1, salt);
        
        // Verify account was created
        assertTrue(address(newAccount) != address(0));
        
        // Check owner
        assertTrue(newAccount.owners(owner1));
        assertEq(newAccount.ownerCount(), 1);
        assertEq(newAccount.ownerList(0), owner1);
    }
    
    function testCreateMultiOwnerAccount() public {
        address[] memory owners = new address[](2);
        owners[0] = owner1;
        owners[1] = owner2;
        
        uint256 salt = uint256(keccak256("test-salt-2"));
        
        // Create account with multiple owners
        AAAccount newAccount = factory.createAccountWithOwners(owners, salt);
        
        // Verify account was created
        assertTrue(address(newAccount) != address(0));
        
        // Check owners
        assertTrue(newAccount.owners(owner1));
        assertTrue(newAccount.owners(owner2));
        assertEq(newAccount.ownerCount(), 2);
        assertEq(newAccount.ownerList(0), owner1);
        assertEq(newAccount.ownerList(1), owner2);
    }
    
    function testAddOwner() public {
        // Create account
        uint256 salt = uint256(keccak256("test-salt-3"));
        AAAccount newAccount = factory.createAccountDirect(owner1, salt);
        
        // Add new owner (owner1 should be able to add owner2)
        vm.prank(owner1);
        newAccount.addOwner(owner2);
        
        // Verify new owner was added
        assertTrue(newAccount.owners(owner2));
        assertEq(newAccount.ownerCount(), 2);
        assertEq(newAccount.ownerList(1), owner2);
    }
    
    function testRemoveOwner() public {
        // Create account with two owners
        address[] memory owners = new address[](2);
        owners[0] = owner1;
        owners[1] = owner2;
        
        uint256 salt = uint256(keccak256("test-salt-4"));
        AAAccount newAccount = factory.createAccountWithOwners(owners, salt);
        
        // Remove owner2
        vm.prank(owner1);
        newAccount.removeOwner(owner2);
        
        // Verify owner2 was removed
        assertFalse(newAccount.owners(owner2));
        assertEq(newAccount.ownerCount(), 1);
        
        // Verify owner1 still exists
        assertTrue(newAccount.owners(owner1));
    }
    
    function testNonOwnerCannotAddOwner() public {
        // Create account with owner1
        uint256 salt = uint256(keccak256("test-salt-5"));
        AAAccount newAccount = factory.createAccountDirect(owner1, salt);
        
        // Try to add owner2 from owner3 (should fail)
        vm.prank(owner3);
        vm.expectRevert("AAAccount: caller is not an owner");
        newAccount.addOwner(owner2);
    }
    
    function testNonOwnerCannotRemoveOwner() public {
        // Create account with owner1
        uint256 salt = uint256(keccak256("test-salt-6"));
        AAAccount newAccount = factory.createAccountDirect(owner1, salt);
        
        // Try to remove owner1 from owner3 (should fail)
        vm.prank(owner3);
        vm.expectRevert("AAAccount: caller is not an owner");
        newAccount.removeOwner(owner1);
    }
    
    function testCannotRemoveLastOwner() public {
        // Create account with single owner
        uint256 salt = uint256(keccak256("test-salt-7"));
        AAAccount newAccount = factory.createAccountDirect(owner1, salt);
        
        // Try to remove the only owner (should fail with "cannot remove self")
        vm.prank(owner1);
        vm.expectRevert("AAAccount: cannot remove self");
        newAccount.removeOwner(owner1);
    }
    
    function testGetAddress() public {
        uint256 salt = uint256(keccak256("test-salt-8"));
        address predictedAddress = factory.getAddress(owner1, salt);
        
        // Create account
        AAAccount newAccount = factory.createAccountDirect(owner1, salt);
        
        // Verify predicted address matches actual address
        assertEq(address(newAccount), predictedAddress);
    }
    
    function testEntryPoint() public {
        uint256 salt = uint256(keccak256("test-salt-9"));
        AAAccount newAccount = factory.createAccountDirect(owner1, salt);
        
        // Verify entryPoint is set correctly
        assertEq(address(newAccount.entryPoint()), address(entryPoint));
    }
    
    function testAccountInitialization() public {
        uint256 salt = uint256(keccak256("test-salt-10"));
        AAAccount newAccount = factory.createAccountDirect(owner1, salt);
        
        // Verify account is initialized
        assertTrue(newAccount.owners(owner1));
        assertEq(newAccount.ownerCount(), 1);
    }

    // =============================================================================
    // ERC-4337 UserOperation Integration Tests
    // =============================================================================

    function testValidateUserOpWithValidSignature() public {
        // Create account
        uint256 salt = uint256(keccak256("test-userOp-1"));
        AAAccount testAccount = factory.createAccountDirect(owner1, salt);
        
        // Fund the account
        vm.deal(address(testAccount), 1 ether);
        
        // Create a simple UserOperation for a transfer
        PackedUserOperation memory userOp = PackedUserOperation({
            sender: address(testAccount),
            nonce: 0,
            initCode: hex"",
            callData: abi.encodeWithSelector(
                AAAccount.execute.selector,
                owner2,
                0.1 ether,
                hex""
            ),
            accountGasLimits: bytes32(abi.encodePacked(uint128(100000), uint128(100000))), // verificationGasLimit + callGasLimit
            preVerificationGas: 21000,
            gasFees: bytes32(abi.encodePacked(uint128(1000000000), uint128(1000000000))), // maxPriorityFeePerGas + maxFeePerGas
            paymasterAndData: hex"",
            signature: hex""
        });
        
        // Get the userOp hash
        bytes32 userOpHash = entryPoint.getUserOpHash(userOp);
        
        // Sign the userOp hash with EIP-191 prefix
        bytes32 hash = MessageHashUtils.toEthSignedMessageHash(userOpHash);
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(owner1Key, hash);
        userOp.signature = abi.encodePacked(r, s, v);
        
        // Validate the signature through EntryPoint
        vm.prank(address(entryPoint));
        uint256 validationData = testAccount.validateUserOp(userOp, userOpHash, 0);
        
        // Should return success (0)
        assertEq(validationData, 0);
    }

    function testValidateUserOpWithInvalidSignature() public {
        // Create account
        uint256 salt = uint256(keccak256("test-userOp-2"));
        AAAccount testAccount = factory.createAccountDirect(owner1, salt);
        
        // Fund the account
        vm.deal(address(testAccount), 1 ether);
        
        // Create a UserOperation
        PackedUserOperation memory userOp = PackedUserOperation({
            sender: address(testAccount),
            nonce: 0,
            initCode: hex"",
            callData: abi.encodeWithSelector(
                AAAccount.execute.selector,
                owner2,
                0.1 ether,
                hex""
            ),
            accountGasLimits: bytes32(abi.encodePacked(uint128(100000), uint128(100000))),
            preVerificationGas: 21000,
            gasFees: bytes32(abi.encodePacked(uint128(1000000000), uint128(1000000000))),
            paymasterAndData: hex"",
            signature: hex"deadbeef" // Invalid signature
        });
        
        // Get the userOp hash
        bytes32 userOpHash = entryPoint.getUserOpHash(userOp);
        
        // Validate the signature through EntryPoint (should fail)
        vm.prank(address(entryPoint));
        uint256 validationData = testAccount.validateUserOp(userOp, userOpHash, 0);
        
        // Should return failure (1)
        assertEq(validationData, 1);
    }

    function testValidateUserOpWithUnauthorizedSigner() public {
        // Create account with owner1
        uint256 salt = uint256(keccak256("test-userOp-3"));
        AAAccount testAccount = factory.createAccountDirect(owner1, salt);
        
        // Fund the account
        vm.deal(address(testAccount), 1 ether);
        
        // Create a UserOperation
        PackedUserOperation memory userOp = PackedUserOperation({
            sender: address(testAccount),
            nonce: 0,
            initCode: hex"",
            callData: abi.encodeWithSelector(
                AAAccount.execute.selector,
                owner3,
                0.1 ether,
                hex""
            ),
            accountGasLimits: bytes32(abi.encodePacked(uint128(100000), uint128(100000))),
            preVerificationGas: 21000,
            gasFees: bytes32(abi.encodePacked(uint128(1000000000), uint128(1000000000))),
            paymasterAndData: hex"",
            signature: hex""
        });
        
        // Get the userOp hash
        bytes32 userOpHash = entryPoint.getUserOpHash(userOp);
        
        // Sign with owner2's key (unauthorized)
        bytes32 hash = MessageHashUtils.toEthSignedMessageHash(userOpHash);
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(owner2Key, hash);
        userOp.signature = abi.encodePacked(r, s, v);
        
        // Validate the signature through EntryPoint (should fail)
        vm.prank(address(entryPoint));
        uint256 validationData = testAccount.validateUserOp(userOp, userOpHash, 0);
        
        // Should return failure (1)
        assertEq(validationData, 1);
    }

    function testUserOperationExecution() public {
        // Create account
        uint256 salt = uint256(keccak256("test-userOp-exec"));
        AAAccount testAccount = factory.createAccountDirect(owner1, salt);
        
        // Fund the account with more ETH for gas
        vm.deal(address(testAccount), 10 ether);
        
        // Record initial balances
        uint256 initialAccountBalance = address(testAccount).balance;
        uint256 initialOwner2Balance = owner2.balance;
        
        // Create a UserOperation for a transfer
        PackedUserOperation memory userOp = PackedUserOperation({
            sender: address(testAccount),
            nonce: 0,
            initCode: hex"",
            callData: abi.encodeWithSelector(
                AAAccount.execute.selector,
                owner2,
                0.1 ether,
                hex""
            ),
            accountGasLimits: bytes32(abi.encodePacked(uint128(500000), uint128(500000))),
            preVerificationGas: 50000,
            gasFees: bytes32(abi.encodePacked(uint128(2000000000), uint128(2000000000))),
            paymasterAndData: hex"",
            signature: hex""
        });
        
        // Get the userOp hash and sign it
        bytes32 userOpHash = entryPoint.getUserOpHash(userOp);
        bytes32 hash = MessageHashUtils.toEthSignedMessageHash(userOpHash);
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(owner1Key, hash);
        userOp.signature = abi.encodePacked(r, s, v);
        
        // Execute the UserOperation through EntryPoint
        PackedUserOperation[] memory userOps = new PackedUserOperation[](1);
        userOps[0] = userOp;
        
        vm.prank(address(this)); // Simulate bundler
        entryPoint.handleOps(userOps, payable(address(this)));
        
        // Verify the transfer occurred (account balance should be less due to gas costs)
        assertTrue(address(testAccount).balance < initialAccountBalance - 0.1 ether, "Account should have less ETH due to gas costs");
        assertEq(owner2.balance, initialOwner2Balance + 0.1 ether, "Owner2 should receive exactly 0.1 ETH");
    }

    function testIsValidSignatureEIP1271() public {
        // Create account
        uint256 salt = uint256(keccak256("test-eip1271"));
        AAAccount testAccount = factory.createAccountDirect(owner1, salt);
        
        // Create a message hash
        bytes32 messageHash = keccak256("test message");
        
        // Sign the message hash directly (not with EIP-191 prefix for this test)
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(owner1Key, messageHash);
        bytes memory signature = abi.encodePacked(r, s, v);
        
        // Test EIP-1271 signature validation
        bytes4 result = testAccount.isValidSignature(messageHash, signature);
        
        // Should return EIP-1271 magic value
        assertEq(result, bytes4(0x1626ba7e));
        
        // Test with unauthorized signer
        (v, r, s) = vm.sign(owner2Key, messageHash);
        signature = abi.encodePacked(r, s, v);
        
        result = testAccount.isValidSignature(messageHash, signature);
        
        // Should return 0 for invalid signature
        assertEq(result, bytes4(0x00000000));
    }

    function testNonceManagement() public {
        // Create account
        uint256 salt = uint256(keccak256("test-nonce"));
        AAAccount testAccount = factory.createAccountDirect(owner1, salt);
        
        // Check initial nonce (should be 0)
        uint256 initialNonce = testAccount.getNonce();
        assertEq(initialNonce, 0);
        
        // Check that EntryPoint manages nonces
        uint256 entryPointNonce = entryPoint.getNonce(address(testAccount), 0);
        assertEq(entryPointNonce, 0);
        
        // After a successful UserOperation, nonce should increment
        // This would be tested in integration with actual EntryPoint execution
        // For now, we just verify the nonce getter works
        assertTrue(initialNonce >= 0);
    }

    function testMultiOwnerSignatureValidation() public {
        // Create multi-owner account
        address[] memory owners = new address[](2);
        owners[0] = owner1;
        owners[1] = owner2;
        
        uint256 salt = uint256(keccak256("test-multi-sig"));
        AAAccount testAccount = factory.createAccountWithOwners(owners, salt);
        
        // Fund the account
        vm.deal(address(testAccount), 1 ether);
        
        // Create UserOperation signed by owner2
        PackedUserOperation memory userOp = PackedUserOperation({
            sender: address(testAccount),
            nonce: 0,
            initCode: hex"",
            callData: abi.encodeWithSelector(
                AAAccount.execute.selector,
                owner3,
                0.1 ether,
                hex""
            ),
            accountGasLimits: bytes32(abi.encodePacked(uint128(100000), uint128(100000))),
            preVerificationGas: 21000,
            gasFees: bytes32(abi.encodePacked(uint128(1000000000), uint128(1000000000))),
            paymasterAndData: hex"",
            signature: hex""
        });
        
        // Sign with owner2 (should be valid)
        bytes32 userOpHash = entryPoint.getUserOpHash(userOp);
        bytes32 hash = MessageHashUtils.toEthSignedMessageHash(userOpHash);
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(owner2Key, hash);
        userOp.signature = abi.encodePacked(r, s, v);
        
        // Validate signature
        vm.prank(address(entryPoint));
        uint256 validationData = testAccount.validateUserOp(userOp, userOpHash, 0);
        
        // Should succeed
        assertEq(validationData, 0);
    }
}
