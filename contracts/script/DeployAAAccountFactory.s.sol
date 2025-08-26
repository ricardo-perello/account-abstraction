// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "forge-std/Script.sol";
import "../src/AAAccountFactory.sol";
import "@account-abstraction/contracts/interfaces/IEntryPoint.sol";

contract DeployAAAccountFactory is Script {
    // Canonical EntryPoint address for v0.7
    address constant ENTRYPOINT_ADDRESS = 0x0000000071727De22E5E9d8BAf0edAc6f37da032;
    
    function run() external {
        vm.startBroadcast();
        
        console.log("Deploying AAAccountFactory...");
        console.log("EntryPoint address:", ENTRYPOINT_ADDRESS);
        
        // Deploy the factory
        AAAccountFactory factory = new AAAccountFactory(IEntryPoint(ENTRYPOINT_ADDRESS));
        
        console.log("AAAccountFactory deployed at:", address(factory));
        console.log("EntryPoint set to:", address(factory.entryPoint()));
        
        vm.stopBroadcast();
    }
}
