// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "forge-std/Script.sol";
import "../lib/account-abstraction/contracts/interfaces/IEntryPoint.sol";

contract CheckPaymasterBalance is Script {
    function run() external view {
        // Sepolia EntryPoint and our new SimplePaymaster
        IEntryPoint entryPoint = IEntryPoint(0x0000000071727De22E5E9d8BAf0edAc6f37da032);
        address paymaster = 0xB0828F3A1F54D52dc91122e6191ffe46da37020f;
        
        console.log("=== SimplePaymaster Balance Check ===");
        console.log("EntryPoint:", address(entryPoint));
        console.log("SimplePaymaster:", paymaster);
        console.log("");
        
        // Check paymaster's deposit in EntryPoint
        uint256 deposit = entryPoint.getDepositInfo(paymaster).deposit;
        console.log("SimplePaymaster deposit in EntryPoint:", deposit);
        console.log("Deposit in ETH:", deposit / 1e18);
        
        // Check paymaster's direct ETH balance
        uint256 paymasterBalance = paymaster.balance;
        console.log("SimplePaymaster direct ETH balance:", paymasterBalance);
        console.log("Balance in ETH:", paymasterBalance / 1e18);
        
        console.log("");
        if (deposit == 0) {
            console.log("WARNING: SimplePaymaster has no deposit in EntryPoint!");
            console.log("This will cause AA33 validation failures.");
        } else if (deposit < 0.01 ether) {
            console.log("WARNING: SimplePaymaster deposit is very low!");
            console.log("Consider adding more funds.");
        } else {
            console.log("SimplePaymaster deposit looks sufficient.");
        }
    }
}
