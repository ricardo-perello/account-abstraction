// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "forge-std/Test.sol";
import "../src/AAAccount.sol";
import "../src/AAAccountFactory.sol";
import "@account-abstraction/contracts/core/EntryPoint.sol";


contract AAAccountTest is Test {
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
        bytes32 salt = keccak256("test-salt-1");
        
        // Create account with single owner
        AAAccount newAccount = factory.createAccount(owner1, salt);
        
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
        
        bytes32 salt = keccak256("test-salt-2");
        
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
        bytes32 salt = keccak256("test-salt-3");
        AAAccount newAccount = factory.createAccount(owner1, salt);
        
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
        
        bytes32 salt = keccak256("test-salt-4");
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
        bytes32 salt = keccak256("test-salt-5");
        AAAccount newAccount = factory.createAccount(owner1, salt);
        
        // Try to add owner2 from owner3 (should fail)
        vm.prank(owner3);
        vm.expectRevert("AAAccount: caller is not an owner");
        newAccount.addOwner(owner2);
    }
    
    function testNonOwnerCannotRemoveOwner() public {
        // Create account with owner1
        bytes32 salt = keccak256("test-salt-6");
        AAAccount newAccount = factory.createAccount(owner1, salt);
        
        // Try to remove owner1 from owner3 (should fail)
        vm.prank(owner3);
        vm.expectRevert("AAAccount: caller is not an owner");
        newAccount.removeOwner(owner1);
    }
    
    function testCannotRemoveLastOwner() public {
        // Create account with single owner
        bytes32 salt = keccak256("test-salt-7");
        AAAccount newAccount = factory.createAccount(owner1, salt);
        
        // Try to remove the only owner (should fail with "cannot remove self")
        vm.prank(owner1);
        vm.expectRevert("AAAccount: cannot remove self");
        newAccount.removeOwner(owner1);
    }
    
    function testGetAddress() public {
        bytes32 salt = keccak256("test-salt-8");
        address predictedAddress = factory.getAddress(owner1, salt);
        
        // Create account
        AAAccount newAccount = factory.createAccount(owner1, salt);
        
        // Verify predicted address matches actual address
        assertEq(address(newAccount), predictedAddress);
    }
    
    function testEntryPoint() public {
        bytes32 salt = keccak256("test-salt-9");
        AAAccount newAccount = factory.createAccount(owner1, salt);
        
        // Verify entryPoint is set correctly
        assertEq(address(newAccount.entryPoint()), address(entryPoint));
    }
    
    function testAccountInitialization() public {
        bytes32 salt = keccak256("test-salt-10");
        AAAccount newAccount = factory.createAccount(owner1, salt);
        
        // Verify account is initialized
        assertTrue(newAccount.owners(owner1));
        assertEq(newAccount.ownerCount(), 1);
    }
}
