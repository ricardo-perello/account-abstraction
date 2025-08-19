// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "forge-std/Script.sol";
import "../src/AAAccountFactory.sol";
import "@account-abstraction/contracts/core/EntryPoint.sol";

contract DeployScript is Script {
    function run() external {
        // Use environment variable or fallback to Anvil's first account
        uint256 deployerKey = vm.envOr("PRIVATE_KEY", uint256(0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80));
        
        vm.startBroadcast(deployerKey);
        
        console.log("Deploying EntryPoint...");
        EntryPoint entryPoint = new EntryPoint();
        console.log("EntryPoint deployed at:", address(entryPoint));
        
        console.log("Deploying AAAccountFactory...");
        AAAccountFactory factory = new AAAccountFactory(IEntryPoint(address(entryPoint)));
        console.log("AAAccountFactory deployed at:", address(factory));
        
        vm.stopBroadcast();
        
        console.log("\n=== Deployment Summary ===");
        console.log("EntryPoint:", address(entryPoint));
        console.log("AAAccountFactory:", address(factory));
        console.log("Deployer:", vm.addr(deployerKey));

    }
}
