// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "forge-std/Script.sol";
import "../lib/account-abstraction/contracts/interfaces/IEntryPoint.sol";

contract CheckPaymasterBalance is Script {
    function run() external view {
        // Sepolia EntryPoint and our paymaster
        IEntryPoint entryPoint = IEntryPoint(0x0000000071727De22E5E9d8BAf0edAc6f37da032);
        address paymaster = 0x3da84818e202009488D2A8e2a3B2f78A6F6321bb;
        
        console.log("=== Paymaster Balance Check ===");
        console.log("EntryPoint:", address(entryPoint));
        console.log("Paymaster:", paymaster);
        console.log("");
        
        // Check paymaster's deposit in EntryPoint
        uint256 deposit = entryPoint.getDepositInfo(paymaster).deposit;
        console.log("Paymaster deposit in EntryPoint:", deposit);
        console.log("Deposit in ETH:", deposit / 1e18);
        
        // Check paymaster's direct ETH balance
        uint256 paymasterBalance = paymaster.balance;
        console.log("Paymaster direct ETH balance:", paymasterBalance);
        console.log("Balance in ETH:", paymasterBalance / 1e18);
        
        console.log("");
        if (deposit == 0) {
            console.log("WARNING: Paymaster has no deposit in EntryPoint!");
            console.log("This will cause AA33 validation failures.");
        } else if (deposit < 0.01 ether) {
            console.log("WARNING: Paymaster deposit is very low!");
            console.log("Consider adding more funds.");
        } else {
            console.log("Paymaster deposit looks sufficient.");
        }
    }
}
