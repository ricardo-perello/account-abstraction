// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "forge-std/Script.sol";
import "../src/SimplePaymaster.sol";
import "../lib/account-abstraction/contracts/interfaces/IEntryPoint.sol";

contract FundSimplePaymaster is Script {
    function run() external {
        vm.startBroadcast();
        
        address paymasterAddress = 0xB0828F3A1F54D52dc91122e6191ffe46da37020f;
        SimplePaymaster paymaster = SimplePaymaster(payable(paymasterAddress));
        
        console.log("=== Funding SimplePaymaster ===");
        console.log("Paymaster address:", paymasterAddress);
        console.log("Deployer address:", msg.sender);
        
        // Fund the paymaster with 0.1 ETH
        uint256 fundingAmount = 0.1 ether;
        console.log("Funding amount:", fundingAmount);
        
        // Deposit directly to EntryPoint for gas sponsorship
        paymaster.deposit{value: fundingAmount}();
        console.log("ETH deposited to EntryPoint");
        
        // Verify the deposit
        IEntryPoint entryPoint = IEntryPoint(0x0000000071727De22E5E9d8BAf0edAc6f37da032);
        uint256 deposit = entryPoint.getDepositInfo(paymasterAddress).deposit;
        console.log("Final deposit in EntryPoint:", deposit);
        
        vm.stopBroadcast();
    }
}
