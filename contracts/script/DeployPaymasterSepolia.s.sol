// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "forge-std/Script.sol";
import "../src/VerifierSignaturePaymaster.sol";
import "../lib/account-abstraction/contracts/interfaces/IEntryPoint.sol";

contract DeployPaymasterSepolia is Script {
    // Sepolia addresses
    address constant ENTRY_POINT = 0x0000000071727De22E5E9d8BAf0edAc6f37da032;
    address constant FACTORY = 0x59bcaa1BB72972Df0446FCe98798076e718E3b61;
    
    // Verifier address - this will be used to verify paymaster signatures
    // You can change this to match the private key you'll use in paymaster-service
    address constant VERIFIER = 0x21D541ef2237b2a63076666651238AC8A7cde752; // From your test wallet
    
    function run() external {
        console.log("=== Deploying VerifierSignaturePaymaster to Sepolia ===");
        console.log("");
        console.log("Network: Sepolia Testnet (Chain ID: 11155111)");
        console.log("EntryPoint:", ENTRY_POINT);
        console.log("Factory:", FACTORY); 
        console.log("Verifier:", VERIFIER);
        console.log("");
        
        vm.startBroadcast();
        
        console.log("Deployer:", msg.sender);
        console.log("Deployer balance:", msg.sender.balance / 1e18, "ETH");
        
        require(msg.sender.balance >= 0.1 ether, "Deployer needs at least 0.1 ETH for deployment and funding");
        
        // Deploy paymaster
        console.log("");
        console.log("Deploying VerifierSignaturePaymaster...");
        VerifierSignaturePaymaster paymaster = new VerifierSignaturePaymaster(
            IEntryPoint(ENTRY_POINT),
            VERIFIER
        );
        
        console.log("Paymaster deployed at:", address(paymaster));
        
        // Fund paymaster with 0.05 ETH for gas sponsorship
        uint256 fundAmount = 0.05 ether;
        console.log("");
        console.log("Funding paymaster with", fundAmount / 1e18, "ETH...");
        
        (bool success, ) = address(paymaster).call{value: fundAmount}("");
        require(success, "Failed to fund paymaster");
        
        // Deposit into EntryPoint
        paymaster.deposit{value: fundAmount}();
        
        console.log("Paymaster funded with", fundAmount / 1e18, "ETH");
        console.log("EntryPoint deposit:", paymaster.getDeposit() / 1e18, "ETH");
        
        vm.stopBroadcast();
        
        console.log("");
        console.log("=== Deployment Complete ===");
        console.log("");
        console.log("Update your configs with:");
        console.log("  - Paymaster address:", address(paymaster));
        console.log("  - Verifier address:", VERIFIER);
        console.log("  - EntryPoint:", ENTRY_POINT);
        console.log("  - Factory:", FACTORY);
        console.log("");
        console.log("Verifier private key to use in paymaster-service:");
        console.log("  0x9ec161507ad1cfd507ae6e6bf012a66d609276782ae64f70ca41174d402d10ae");
        console.log("");
        console.log("SECURITY: This is a test key - NEVER use on mainnet!");
        console.log("");
        console.log("Ready to test sponsored transactions!");
    }
}
