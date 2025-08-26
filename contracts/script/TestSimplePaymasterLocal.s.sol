// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "forge-std/Script.sol";
import "../src/SimplePaymaster.sol";
import "../lib/account-abstraction/contracts/core/EntryPoint.sol";
import "../lib/account-abstraction/contracts/interfaces/IEntryPoint.sol";

contract TestSimplePaymasterLocal is Script {
    // Deployed contracts
    EntryPoint public entryPoint;
    SimplePaymaster public paymaster;
    
    // Test accounts
    address public deployer;
    address public user;
    
    // Private keys for signing
    uint256 public deployerKey;
    uint256 public userKey;
    
    function run() external {
        console.log("Starting SimplePaymaster Local Test");
        console.log("===================================");
        
        setupAccounts();
        deployContracts();
        testBasicFunctionality();
        testPaymasterValidation();
        
        console.log("Local test completed successfully!");
    }
    
    function setupAccounts() internal {
        console.log("Setting up test accounts...");
        
        // Use anvil's default accounts
        deployerKey = 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80;
        userKey = 0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d;
        
        deployer = vm.addr(deployerKey);
        user = vm.addr(userKey);
        
        console.log("Deployer:", deployer);
        console.log("User:", user);
    }
    
    function deployContracts() internal {
        console.log("Deploying contracts...");
        
        vm.startBroadcast(deployerKey);
        
        // Deploy EntryPoint
        entryPoint = new EntryPoint();
        console.log("EntryPoint deployed at:", address(entryPoint));
        
        // Deploy SimplePaymaster
        paymaster = new SimplePaymaster(entryPoint, 0.1 ether);
        console.log("SimplePaymaster deployed at:", address(paymaster));
        
        vm.stopBroadcast();
    }
    
    function testBasicFunctionality() internal {
        console.log("Testing basic functionality...");
        
        vm.startBroadcast(deployerKey);
        
        // Test deposit
        paymaster.deposit{value: 1 ether}();
        console.log("Paymaster deposit:", paymaster.getDeposit());
        
        // Test max gas cost
        console.log("Max gas cost:", paymaster.maxGasCost());
        
        // Test isActive
        console.log("Paymaster active:", paymaster.isActive());
        
        // Test toggle
        paymaster.togglePaymaster();
        console.log("Paymaster active after toggle:", paymaster.isActive());
        
        // Toggle back
        paymaster.togglePaymaster();
        console.log("Paymaster active after second toggle:", paymaster.isActive());
        
        vm.stopBroadcast();
    }
    
    function testPaymasterValidation() internal {
        console.log("Testing paymaster validation...");
        
        // Create a mock UserOperation
        bytes memory paymasterAndData = abi.encodePacked(
            address(paymaster),
            uint128(50000), // verificationGasLimit
            uint128(20000), // postOpGasLimit
            "" // empty data
        );
        
        // Test validation by calling from EntryPoint context
        vm.startBroadcast(deployerKey);
        
        // Test the EntryPoint interface by trying to deploy with it
        console.log("Testing EntryPoint interface by deployment...");
        console.log("EntryPoint address:", address(entryPoint));
        console.log("EntryPoint code length:", address(entryPoint).code.length);
        
        vm.stopBroadcast();
        
        // Now test the actual validation by impersonating the EntryPoint
        console.log("Testing paymaster validation by impersonating EntryPoint...");
        
        // Impersonate the EntryPoint to call validatePaymasterUserOp
        vm.prank(address(entryPoint));
        
        try paymaster.validatePaymasterUserOp(
            PackedUserOperation({
                sender: user,
                nonce: 0,
                initCode: "",
                callData: "",
                accountGasLimits: bytes32(abi.encodePacked(uint128(100000), uint128(100000))),
                preVerificationGas: 21000,
                gasFees: bytes32(abi.encodePacked(uint128(1000000000), uint128(10000000000))),
                paymasterAndData: paymasterAndData,
                signature: ""
            }),
            bytes32(0), // userOpHash
            0.05 ether // maxCost
        ) returns (bytes memory context, uint256 validationData) {
            console.log("Validation successful!");
            console.log("Context length:", context.length);
            console.log("Validation data:", validationData);
        } catch Error(string memory reason) {
            console.log("Validation failed with reason:", reason);
        } catch (bytes memory lowLevelData) {
            console.log("Validation failed with low-level error");
            console.logBytes(lowLevelData);
        }
    }
}
