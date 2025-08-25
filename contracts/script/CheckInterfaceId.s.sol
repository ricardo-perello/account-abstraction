// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "forge-std/Script.sol";
import "../lib/account-abstraction/contracts/interfaces/IEntryPoint.sol";

contract CheckInterfaceId is Script {
    function run() external pure {
        console.log("=== Interface ID Investigation ===");
        console.log("");
        
        // Get the interface ID for our imported IEntryPoint
        bytes4 ourInterfaceId = type(IEntryPoint).interfaceId;
        console.log("Our IEntryPoint interface ID:", vm.toString(ourInterfaceId));
        console.log("Our IEntryPoint interface ID (hex):", vm.toString(uint256(uint32(ourInterfaceId))));
        
        // The interface ID that was failing in the original error
        bytes4 failingId = 0x989ccc58;
        console.log("Failing interface ID from logs:", vm.toString(failingId)); 
        
        console.log("");
        console.log("Match?", ourInterfaceId == failingId ? "YES" : "NO");
    }
}
