// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "forge-std/Script.sol";
import "../src/AAAccountFactory.sol";
import "../src/AAAccount.sol";
import "@account-abstraction/contracts/core/EntryPoint.sol";

contract InteractScript is Script {
    // Anvil account 0 private key (standard)
    uint256 constant PRIVATE_KEY = 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80;
    
    // Anvil account 1 private key
    uint256 constant OWNER1_KEY = 0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d;
    
    // Anvil account 2 private key  
    uint256 constant OWNER2_KEY = 0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a;
    
    function run() external {
        // Use the deployed contract addresses
        address entryPointAddr = 0x5FbDB2315678afecb367f032d93F642f64180aa3;
        address factoryAddr = 0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512;
        
        console.log("=== AA Contract Interaction Demo ===");
        console.log("EntryPoint:", entryPointAddr);
        console.log("Factory:", factoryAddr);
        
        vm.startBroadcast(PRIVATE_KEY);
        
        // Get contract instances
        EntryPoint entryPoint = EntryPoint(payable(entryPointAddr));
        AAAccountFactory factory = AAAccountFactory(factoryAddr);
        
        // Create a single owner account
        console.log("\n--- Creating Single Owner Account ---");
        uint256 salt1 = uint256(keccak256("demo-salt-1"));
        address predictedAddr1 = factory.getAddress(vm.addr(OWNER1_KEY), salt1);
        console.log("Predicted address:", predictedAddr1);
        
        AAAccount account1 = factory.createAccount(vm.addr(OWNER1_KEY), salt1);
        console.log("Account created at:", address(account1));
        console.log("Owner count:", account1.ownerCount());
        console.log("Is owner1 owner?", account1.owners(vm.addr(OWNER1_KEY)));
        
        // Create a multi-owner account
        console.log("\n--- Creating Multi-Owner Account ---");
        address[] memory owners = new address[](2);
        owners[0] = vm.addr(OWNER1_KEY);
        owners[1] = vm.addr(OWNER2_KEY);
        
        uint256 salt2 = uint256(keccak256("demo-salt-2"));
        address predictedAddr2 = factory.getAddressWithOwners(owners, salt2);
        console.log("Predicted address:", predictedAddr2);
        
        AAAccount account2 = factory.createAccountWithOwners(owners, salt2);
        console.log("Multi-owner account created at:", address(account2));
        console.log("Owner count:", account2.ownerCount());
        console.log("Is owner1 owner?", account2.owners(vm.addr(OWNER1_KEY)));
        console.log("Is owner2 owner?", account2.owners(vm.addr(OWNER2_KEY)));
        
        vm.stopBroadcast();
        
        // Test account interactions (using different signer)
        vm.startBroadcast(OWNER1_KEY);
        
        console.log("\n--- Testing Account Interactions ---");
        
        // Add a new owner to account1
        console.log("Adding owner2 to account1...");
        account1.addOwner(vm.addr(OWNER2_KEY));
        console.log("New owner count:", account1.ownerCount());
        console.log("Is owner2 now owner?", account1.owners(vm.addr(OWNER2_KEY)));
        
        // Remove owner2 from account1
        console.log("Removing owner2 from account1...");
        account1.removeOwner(vm.addr(OWNER2_KEY));
        console.log("Final owner count:", account1.ownerCount());
        console.log("Is owner2 still owner?", account1.owners(vm.addr(OWNER2_KEY)));
        
        vm.stopBroadcast();
        
        console.log("\n=== Demo Complete ===");
        console.log("Single owner account:", address(account1));
        console.log("Multi-owner account:", address(account2));
    }
}
