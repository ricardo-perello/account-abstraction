// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "forge-std/Script.sol";
import "../src/SimplePaymaster.sol";
import "../lib/account-abstraction/contracts/core/EntryPoint.sol";
import "../lib/account-abstraction/contracts/interfaces/IEntryPoint.sol";

contract DeploySimplePaymaster is Script {
    // EntryPoint address - canonical ERC-4337 EntryPoint
    address public constant ENTRYPOINT_ADDRESS = address(0x0000000071727De22E5E9d8BAf0edAc6f37da032);
    
    // Maximum gas cost to sponsor per operation (0.01 ETH)
    uint256 public constant MAX_GAS_COST = 0.01 ether;
    
    function run() external {
        console.log("Deploying SimplePaymaster...");
        
        vm.startBroadcast();
        
        // Deploy SimplePaymaster with canonical EntryPoint
        SimplePaymaster paymaster = new SimplePaymaster(
            IEntryPoint(ENTRYPOINT_ADDRESS),
            MAX_GAS_COST
        );
        
        console.log("SimplePaymaster deployed at:", address(paymaster));
        console.log("Max Gas Cost:", vm.toString(MAX_GAS_COST / 1e18), "ETH");
        
        // Note: Paymaster needs to be funded separately after deployment
        console.log("Next steps:");
        console.log("1. Fund the paymaster with ETH for gas sponsorship");
        console.log("2. Call paymaster.deposit() to add funds to EntryPoint");
        
        vm.stopBroadcast();
    }
}
