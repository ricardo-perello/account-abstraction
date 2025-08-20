// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "forge-std/Script.sol";
import "../src/AAAccountFactory.sol";

contract DeploySepoliaScript is Script {
    // EntryPoint v0.7+ address for Sepolia
    address constant ENTRY_POINT_V07 = 0x0000000071727De22E5E9d8BAf0edAc6f37da032;
    
    function run() external {
        // Use the private key from environment variable PRIVATE_KEY
        uint256 deployerKey = vm.envOr("PRIVATE_KEY", uint256(0));
        require(deployerKey != 0, "PRIVATE_KEY not found in environment");
        address deployer = vm.addr(deployerKey);
        
        console.log("=== Sepolia Deployment ===");
        console.log("Deployer:", deployer);
        console.log("Using EntryPoint v0.7+:", ENTRY_POINT_V07);
        
        // Check deployer balance
        console.log("Deployer ETH balance:", deployer.balance / 1e18, "ETH");
        require(deployer.balance > 0.01 ether, "Insufficient ETH for deployment");
        
        vm.startBroadcast(deployerKey);
        
        console.log("Deploying AAAccountFactory...");
        AAAccountFactory factory = new AAAccountFactory(IEntryPoint(ENTRY_POINT_V07));
        console.log("AAAccountFactory deployed at:", address(factory));
        
        vm.stopBroadcast();
        
        console.log("\n=== Deployment Summary ===");
        console.log("Network: Sepolia Testnet");
        console.log("EntryPoint (v0.7+):", ENTRY_POINT_V07);
        console.log("AAAccountFactory:", address(factory));
        console.log("Deployer:", deployer);
        console.log("Gas used: See transaction receipt");
        
        // Verify the factory is correctly configured
        console.log("\n=== Verification ===");
        console.log("Factory EntryPoint:", address(factory.entryPoint()));
        require(address(factory.entryPoint()) == ENTRY_POINT_V07, "EntryPoint mismatch!");
        console.log("SUCCESS: Factory correctly configured with EntryPoint v0.7+");
    }
}
