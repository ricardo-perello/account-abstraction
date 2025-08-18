// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "forge-std/Test.sol";
import "../src/SmartAccount.sol";
import "../src/interfaces/IEntryPoint.sol";

contract MockEntryPoint is IEntryPoint {
    function handleOps(UserOperation[] calldata ops, address payable beneficiary) external {}
    function simulateValidation(UserOperation calldata userOp) external {}
    function getUserOpHash(UserOperation calldata userOp) external pure returns (bytes32) {
        return keccak256(abi.encode(userOp));
    }
}

contract SmartAccountTest is Test {
    SmartAccount public smartAccount;
    MockEntryPoint public entryPoint;
    
    address public owner1 = address(0x1);
    address public owner2 = address(0x2);
    address public owner3 = address(0x3);
    address public nonOwner = address(0x4);
    
    function setUp() public {
        entryPoint = new MockEntryPoint();
        vm.prank(owner1);
        smartAccount = new SmartAccount(address(entryPoint));
    }
    
    // Owner Management Tests
    
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
    
    // Transaction Execution Tests
    
    function test_execute_success() public {
        address target = address(0x123);
        uint256 value = 0;
        bytes memory data = hex"1234";
        
        vm.prank(owner1);
        smartAccount.execute(target, value, data);
        
        // Note: In a real test, you'd want to verify the actual call was made
        // This is a basic test that the function doesn't revert
    }
    
    function test_execute_fails_if_not_owner() public {
        vm.prank(nonOwner);
        vm.expectRevert("SmartAccount: caller is not an owner");
        smartAccount.execute(address(0x123), 0, hex"1234");
    }
    
    function test_execute_fails_if_invalid_target() public {
        vm.prank(owner1);
        vm.expectRevert("SmartAccount: invalid target");
        smartAccount.execute(address(0), 0, hex"1234");
    }
    
    function test_executeBatch_success() public {
        address[] memory targets = new address[](2);
        targets[0] = address(0x123);
        targets[1] = address(0x456);
        
        uint256[] memory values = new uint256[](2);
        values[0] = 0;
        values[1] = 0;
        
        bytes[] memory datas = new bytes[](2);
        datas[0] = hex"1234";
        datas[1] = hex"5678";
        
        vm.prank(owner1);
        smartAccount.executeBatch(targets, values, datas);
    }
    
    function test_executeBatch_fails_if_not_owner() public {
        address[] memory targets = new address[](1);
        targets[0] = address(0x123);
        
        uint256[] memory values = new uint256[](1);
        values[0] = 0;
        
        bytes[] memory datas = new bytes[](1);
        datas[0] = hex"1234";
        
        vm.prank(nonOwner);
        vm.expectRevert("SmartAccount: caller is not an owner");
        smartAccount.executeBatch(targets, values, datas);
    }
    
    function test_executeBatch_fails_if_array_length_mismatch() public {
        address[] memory targets = new address[](2);
        targets[0] = address(0x123);
        targets[1] = address(0x456);
        
        uint256[] memory values = new uint256[](1);
        values[0] = 0;
        
        bytes[] memory datas = new bytes[](2);
        datas[0] = hex"1234";
        datas[1] = hex"5678";
        
        vm.prank(owner1);
        vm.expectRevert("SmartAccount: array length mismatch");
        smartAccount.executeBatch(targets, values, datas);
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
