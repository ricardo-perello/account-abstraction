// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "forge-std/Script.sol";
import "../src/SimplePaymaster.sol";

contract StakeSimplePaymaster is Script {
    function run() external {
        vm.startBroadcast();
        
        address paymasterAddress = 0xB0828F3A1F54D52dc91122e6191ffe46da37020f;
        SimplePaymaster paymaster = SimplePaymaster(payable(paymasterAddress));
        
        console.log("=== Adding Stake to SimplePaymaster ===");
        console.log("Paymaster address:", paymasterAddress);
        console.log("Deployer address:", msg.sender);
        
        // Add stake with 1 day unstake delay (86400 seconds)
        uint32 unstakeDelay = 86400;
        uint256 stakeAmount = 0.1 ether;
        
        console.log("Stake amount:", stakeAmount);
        console.log("Unstake delay:", unstakeDelay, "seconds (1 day)");
        
        // Add stake to the paymaster
        paymaster.addStake{value: stakeAmount}(unstakeDelay);
        console.log("Stake added successfully");
        
        vm.stopBroadcast();
    }
}
