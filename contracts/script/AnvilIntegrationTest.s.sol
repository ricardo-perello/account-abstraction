// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "forge-std/Script.sol";
import "../src/AAAccount.sol";
import "../src/AAAccountFactory.sol";
import "../src/VerifierSignaturePaymaster.sol";
import "../lib/account-abstraction/contracts/core/EntryPoint.sol";
import "../lib/account-abstraction/contracts/interfaces/PackedUserOperation.sol";

contract AnvilIntegrationTest is Script {
    // Deployed contracts
    EntryPoint public entryPoint;
    AAAccountFactory public factory;
    AAAccount public account;
    VerifierSignaturePaymaster public paymaster;
    
    // Test accounts
    address public deployer;
    address public bundler;
    address public accountOwner;
    address public verifier;
    address public recipient;
    
    // Private keys for signing
    uint256 public deployerKey;
    uint256 public bundlerKey;
    uint256 public accountOwnerKey;
    uint256 public verifierKey;
    
    function run() external {
        console.log("Starting Anvil Integration Test");
        console.log("=====================================");
        
        setupAccounts();
        deployContracts();
        setupPaymaster();
        testGaslessAccountCreation();
        testGaslessTransaction();
        
        console.log("Integration test completed successfully!");
    }
    
    function setupAccounts() internal {
        console.log("Setting up test accounts...");
        
        // Use anvil's default accounts
        deployerKey = 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80;
        bundlerKey = 0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d;
        accountOwnerKey = 0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a;
        verifierKey = 0x7c852118294e51e653712a81e05800f419141751be58f605c371e15141b007a6;
        
        deployer = vm.addr(deployerKey);
        bundler = vm.addr(bundlerKey);
        accountOwner = vm.addr(accountOwnerKey);
        verifier = vm.addr(verifierKey);
        recipient = address(0x1234567890123456789012345678901234567890);
        
        console.log("Deployer:", deployer);
        console.log("Bundler:", bundler);
        console.log("Account Owner:", accountOwner);
        console.log("Verifier:", verifier);
        console.log("Recipient:", recipient);
    }
    
    function deployContracts() internal {
        console.log("Deploying contracts...");
        
        vm.startBroadcast(deployerKey);
        
        // Deploy EntryPoint
        entryPoint = new EntryPoint();
        console.log("EntryPoint deployed at:", address(entryPoint));
        
        // Deploy AAAccountFactory
        factory = new AAAccountFactory(entryPoint);
        console.log("AAAccountFactory deployed at:", address(factory));
        
        // Deploy Paymaster
        paymaster = new VerifierSignaturePaymaster(entryPoint, verifier);
        console.log("Paymaster deployed at:", address(paymaster));
        
        vm.stopBroadcast();
    }
    
    function setupPaymaster() internal {
        console.log("Setting up paymaster...");
        
        vm.startBroadcast(deployerKey);
        
        // Fund paymaster with 10 ETH
        vm.deal(address(paymaster), 10 ether);
        paymaster.deposit{value: 10 ether}();
        
        console.log("Paymaster funded with 10 ETH");
        console.log("Paymaster deposit:", paymaster.getDeposit());
        
        vm.stopBroadcast();
    }
    
    function testGaslessAccountCreation() internal {
        console.log("Testing gasless account creation...");
        
        // Predict account address
        address predictedAccount = factory.getAddress(accountOwner, 0);
        console.log("Predicted account address:", predictedAccount);
        
        // Create UserOperation for account creation
        PackedUserOperation memory userOp = createAccountCreationUserOp(predictedAccount);
        console.log("Initial UserOp created");
        console.log("  - verificationGasLimit:", uint128(uint256(userOp.accountGasLimits) >> 128));
        console.log("  - callGasLimit:", uint128(uint256(userOp.accountGasLimits)));
        console.log("  - preVerificationGas:", userOp.preVerificationGas);
        console.log("  - initCode length:", userOp.initCode.length);
        
        // Step 1: Create UserOp WITHOUT paymaster data first
        console.log("Creating UserOp without paymaster data...");
        
        // Step 2: Add paymaster data FIRST (before signing)
        console.log("Adding paymaster data...");
        userOp = addPaymasterDataWithHash(userOp, bytes32(0)); // finalUserOpHash not used anymore
        console.log("Paymaster data added");
        console.log("  - paymasterAndData length:", userOp.paymasterAndData.length);
        
        // Step 3: NOW sign the UserOp WITH paymaster data
        bytes32 finalUserOpHash = entryPoint.getUserOpHash(userOp);
        console.log("Signing UserOp with paymaster data...");
        console.log("  - final UserOp hash:", vm.toString(finalUserOpHash));
        userOp = signUserOperationWithHash(userOp, finalUserOpHash);
        console.log("UserOperation signed");
        console.log("  - signature length:", userOp.signature.length);
        
        // Step 4: Verify the final hash matches what we signed
        bytes32 verificationHash = entryPoint.getUserOpHash(userOp);
        console.log("Verification - final UserOp hash:", vm.toString(verificationHash));
        require(finalUserOpHash == verificationHash, "Hash mismatch after signing");
        
        console.log("Account owner balance before:", accountOwner.balance);
        console.log("Predicted account balance before:", predictedAccount.balance);
        console.log("Paymaster balance before:", address(paymaster).balance);
        console.log("Bundler balance before:", bundler.balance);
        
        // Submit via bundler (gasless account creation!)
        vm.startBroadcast(bundlerKey);
        
        PackedUserOperation[] memory ops = new PackedUserOperation[](1);
        ops[0] = userOp;
        
        console.log("Submitting gasless account creation...");
        console.log("Gas limit for handleOps call: unlimited (forge script)");
        
        try entryPoint.handleOps(ops, payable(bundler)) {
            console.log("SUCCESS: handleOps completed!");
        } catch Error(string memory reason) {
            console.log("ERROR in handleOps:", reason);
            revert(reason);
        } catch (bytes memory lowLevelData) {
            console.log("LOW-LEVEL ERROR in handleOps");
            console.logBytes(lowLevelData);
            revert("handleOps failed with low-level error");
        }
        
        vm.stopBroadcast();
        
        // Verify account was created
        account = AAAccount(payable(predictedAccount));
        uint256 codeLength = address(account).code.length;
        console.log("Account code length after:", codeLength);
        require(codeLength > 0, "Account not deployed");
        console.log("SANITY CHECK: Account has", codeLength, "bytes of code - deployment successful!");
        console.log("Smart account created gaslessly at:", address(account));
        console.log("Account owner balance after:", accountOwner.balance);
        console.log("Account balance after:", address(account).balance);
        console.log("Paymaster balance after:", address(paymaster).balance);
        console.log("Bundler balance after:", bundler.balance);
    }
    
    function testGaslessTransaction() internal {
        console.log("Testing gasless transaction...");
        
        uint256 transferAmount = 0.1 ether;
        uint256 recipientBalanceBefore = recipient.balance;
        
        console.log("Recipient balance before:", recipientBalanceBefore);
        console.log("Transfer amount:", transferAmount);
        
        // Fund the account so it can send ETH
        vm.deal(address(account), 1 ether);
        
        // Create UserOperation for ETH transfer
        PackedUserOperation memory userOp = createUserOperation(
            address(account),
            recipient,
            transferAmount,
            ""
        );
        
        // Step 1: Create UserOp WITHOUT paymaster data first
        console.log("Creating UserOp without paymaster data...");
        
        // Step 2: Add paymaster data FIRST (before signing)
        console.log("Adding paymaster data...");
        userOp = addPaymasterDataWithHash(userOp, bytes32(0)); // finalUserOpHash not used anymore
        
        // Step 3: NOW sign the UserOp WITH paymaster data
        bytes32 finalUserOpHash = entryPoint.getUserOpHash(userOp);
        console.log("Signing UserOp with paymaster data...");
        userOp = signUserOperationWithHash(userOp, finalUserOpHash);
        
        // Submit via bundler (EOA acting as bundler)
        vm.startBroadcast(bundlerKey);
        
        PackedUserOperation[] memory ops = new PackedUserOperation[](1);
        ops[0] = userOp;
        
        console.log("Submitting UserOperation via bundler...");
        entryPoint.handleOps(ops, payable(bundler));
        
        vm.stopBroadcast();
        
        // Verify transaction succeeded
        uint256 recipientBalanceAfter = recipient.balance;
        console.log("Recipient balance after:", recipientBalanceAfter);
        
        require(
            recipientBalanceAfter == recipientBalanceBefore + transferAmount,
            "Transfer failed"
        );
        
        console.log("Gasless transaction successful!");
        console.log("Amount transferred:", transferAmount);
    }
    
    function createAccountCreationUserOp(address predictedAccount) 
        internal view returns (PackedUserOperation memory) {
        
        // Create initCode for account creation
        bytes memory initCode = abi.encodePacked(
            address(factory),
            abi.encodeWithSelector(
                AAAccountFactory.createAccount.selector,
                accountOwner,
                0 // salt
            )
        );
        
        return PackedUserOperation({
            sender: predictedAccount,
            nonce: 0, // First nonce for new account
            initCode: initCode,
            callData: "", // No call after creation
            accountGasLimits: bytes32(abi.encodePacked(uint128(500000), uint128(50000))),
            preVerificationGas: 21000,
            gasFees: bytes32(abi.encodePacked(uint128(1000000000), uint128(10000000000))),
            paymasterAndData: "",
            signature: ""
        });
    }
    
    function createUserOperation(
        address sender,
        address target,
        uint256 value,
        bytes memory data
    ) internal view returns (PackedUserOperation memory) {
        // Get nonce
        uint256 nonce = entryPoint.getNonce(sender, 0);
        
        // Encode call data for account execution
        bytes memory callData = abi.encodeWithSelector(
            AAAccount.execute.selector,
            target,
            value,
            data
        );
        
        return PackedUserOperation({
            sender: sender,
            nonce: nonce,
            initCode: "",
            callData: callData,
            accountGasLimits: bytes32(abi.encodePacked(uint128(100000), uint128(100000))), // Increased for ETH transfers
            preVerificationGas: 21000,
            gasFees: bytes32(abi.encodePacked(uint128(1000000000), uint128(10000000000))),
            paymasterAndData: "",
            signature: ""
        });
    }
    
    function _calculateMaxCost(PackedUserOperation memory userOp, uint128 paymasterVerificationGasLimit, uint128 paymasterPostOpGasLimit) 
        internal view returns (uint256) {
        console.log("  - _calculateMaxCost: starting calculation");
        
        // Extract values directly from the packed fields to avoid abi.decode issues
        uint128 verificationGasLimit = uint128(uint256(userOp.accountGasLimits) >> 128);
        uint128 callGasLimit = uint128(uint256(userOp.accountGasLimits));
        uint128 maxPriorityFeePerGas = uint128(uint256(userOp.gasFees) >> 128);
        uint128 maxFeePerGas = uint128(uint256(userOp.gasFees));
        
        console.log("  - verificationGasLimit:", verificationGasLimit);
        console.log("  - callGasLimit:", callGasLimit);
        console.log("  - maxPriorityFeePerGas:", maxPriorityFeePerGas);
        console.log("  - maxFeePerGas:", maxFeePerGas);
        console.log("  - preVerificationGas:", userOp.preVerificationGas);
        
        uint256 totalGas = verificationGasLimit + callGasLimit + 
                          paymasterVerificationGasLimit + paymasterPostOpGasLimit + 
                          userOp.preVerificationGas;
        console.log("  - totalGas:", totalGas);
        
        uint256 result = totalGas * maxFeePerGas;
        console.log("  - result:", result);
        
        return result;
    }
    
    function addPaymasterDataWithHash(PackedUserOperation memory userOp, bytes32 finalUserOpHash) 
        internal view returns (PackedUserOperation memory) {
        
        uint64 validUntil = uint64(block.timestamp + 3600);
        uint64 validAfter = 0; // Not used in this implementation
        uint128 paymasterVerificationGasLimit = 50000;
        uint128 paymasterPostOpGasLimit = 50000;
        
        console.log("Creating paymaster data...");
        console.log("  - validUntil:", validUntil);
        console.log("  - validAfter:", validAfter);
        console.log("  - paymasterVerificationGasLimit:", paymasterVerificationGasLimit);
        console.log("  - paymasterPostOpGasLimit:", paymasterPostOpGasLimit);
        
        // Step 1: Get the paymaster-specific digest (excludes paymasterAndData)
        bytes32 paymasterDigest = paymaster.getPaymasterDigest(userOp, validUntil, validAfter);
        console.log("  - paymaster digest:", vm.toString(paymasterDigest));
        
        // Step 2: Sign the paymaster digest with verifier key
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(verifierKey, paymasterDigest);
        console.log("  - signature v:", v);
        console.log("  - signature r:", vm.toString(r));
        console.log("  - signature s:", vm.toString(s));
        
        // Step 3: Create paymasterAndData with real signature
        userOp.paymasterAndData = abi.encodePacked(
            address(paymaster), 
            paymasterVerificationGasLimit, 
            paymasterPostOpGasLimit,
            abi.encodePacked(r, s, v), // Real signature
            validUntil,
            validAfter
        );
        
        console.log("  - paymasterAndData length:", userOp.paymasterAndData.length);
        
        return userOp;
    }
    
    function signUserOperationWithHash(PackedUserOperation memory userOp, bytes32 baseUserOpHash) 
        internal view returns (PackedUserOperation memory) {
        
        console.log("  - signing with baseUserOpHash:", vm.toString(baseUserOpHash));
        
        // Sign with account owner key using the BASE hash
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(accountOwnerKey, baseUserOpHash);
        userOp.signature = abi.encodePacked(r, s, v);
        
        return userOp;
    }
}
