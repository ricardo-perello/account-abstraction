// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "forge-std/Script.sol";
import "../src/VerifierSignaturePaymaster.sol";

contract DeployPaymasterScript is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address entryPoint = vm.envAddress("ENTRY_POINT");
        address verifier = vm.envAddress("VERIFIER_ADDRESS");
        
        vm.startBroadcast(deployerPrivateKey);
        
        VerifierSignaturePaymaster paymaster = new VerifierSignaturePaymaster(
            IEntryPoint(entryPoint),
            verifier
        );
        
        vm.stopBroadcast();
        
        console.log("Paymaster deployed at:", address(paymaster));
        console.log("EntryPoint:", entryPoint);
        console.log("Verifier:", verifier);
    }
}
