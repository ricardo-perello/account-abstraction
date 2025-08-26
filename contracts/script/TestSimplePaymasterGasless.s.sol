// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "forge-std/Script.sol";
import "../src/SimplePaymaster.sol";
import "../src/AAAccount.sol";
import "../src/AAAccountFactory.sol";
import "../lib/account-abstraction/contracts/core/EntryPoint.sol";
import "../lib/account-abstraction/contracts/interfaces/PackedUserOperation.sol";
import "../lib/account-abstraction/contracts/core/UserOperationLib.sol";
import "../lib/account-abstraction/contracts/core/Helpers.sol";

contract TestSimplePaymasterGasless is Script {
    // Deployed contracts
    EntryPoint public entryPoint;
    SimplePaymaster public paymaster;
    AAAccountFactory public factory;
    AAAccount public account;
    
    // Test accounts
    address public deployer;
    address public bundler;
    address public accountOwner;
    address public recipient;
    
    // Private keys for signing
    uint256 public deployerKey;
    uint256 public bundlerKey;
    uint256 public accountOwnerKey;
    
    function run() external {
        console.log("Starting SimplePaymaster Gasless Test");
        console.log("=====================================");
        
        setupAccounts();
        deployContracts();
        setupPaymaster();
        testGaslessAccountCreation();
        testGaslessTransaction();
        
        console.log("Gasless test completed successfully!");
    }
    
    function setupAccounts() internal {
        console.log("Setting up test accounts...");
        
        // Use anvil's default accounts
        deployerKey = 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80;
        bundlerKey = 0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d;
        accountOwnerKey = 0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a;
        
        deployer = vm.addr(deployerKey);
        bundler = vm.addr(bundlerKey);
        accountOwner = vm.addr(accountOwnerKey);
        recipient = address(0x1234567890123456789012345678901234567890);
        
        console.log("Deployer:", deployer);
        console.log("Bundler:", bundler);
        console.log("Account Owner:", accountOwner);
        console.log("Recipient:", recipient);
    }
    
    function deployContracts() internal {
        console.log("Deploying contracts...");
        
        vm.startBroadcast(deployerKey);
        
        // Deploy EntryPoint
        entryPoint = new EntryPoint();
        console.log("EntryPoint deployed at:", address(entryPoint));
        
        // Deploy SimplePaymaster
        paymaster = new SimplePaymaster(entryPoint, 0.1 ether);
        console.log("SimplePaymaster deployed at:", address(paymaster));
        
        // Deploy AAAccountFactory
        factory = new AAAccountFactory(entryPoint);
        console.log("AAAccountFactory deployed at:", address(factory));
        
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
        
        // Add paymaster data
        console.log("Adding paymaster data...");
        userOp = addPaymasterData(userOp);
        console.log("Paymaster data added");
        console.log("  - paymasterAndData length:", userOp.paymasterAndData.length);
        
        // Sign the UserOp
        bytes32 userOpHash = entryPoint.getUserOpHash(userOp);
        console.log("Signing UserOp...");
        console.log("  - UserOp hash:", vm.toString(userOpHash));
        userOp = signUserOperation(userOp, userOpHash);
        console.log("UserOperation signed");
        console.log("  - signature length:", userOp.signature.length);
        
        console.log("Account owner balance before:", accountOwner.balance);
        console.log("Predicted account balance before:", predictedAccount.balance);
        console.log("Paymaster balance before:", address(paymaster).balance);
        console.log("Bundler balance before:", bundler.balance);
        
        // Submit via bundler (gasless account creation!)
        vm.startBroadcast(bundlerKey);
        
        PackedUserOperation[] memory ops = new PackedUserOperation[](1);
        ops[0] = userOp;
        
        console.log("Submitting gasless account creation...");
        
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
        
        // Add paymaster data
        console.log("Adding paymaster data...");
        userOp = addPaymasterData(userOp);
        
        // Sign the UserOp
        bytes32 userOpHash = entryPoint.getUserOpHash(userOp);
        console.log("Signing UserOp...");
        userOp = signUserOperation(userOp, userOpHash);
        
        // Submit via bundler
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
            accountGasLimits: bytes32(abi.encodePacked(uint128(100000), uint128(100000))),
            preVerificationGas: 21000,
            gasFees: bytes32(abi.encodePacked(uint128(1000000000), uint128(10000000000))),
            paymasterAndData: "",
            signature: ""
        });
    }
    
    function addPaymasterData(PackedUserOperation memory userOp) 
        internal view returns (PackedUserOperation memory) {
        
        uint64 validUntil = uint64(block.timestamp + 3600);
        uint64 validAfter = 0;
        uint128 paymasterVerificationGasLimit = 50000;
        uint128 paymasterPostOpGasLimit = 20000;
        
        console.log("Creating paymaster data...");
        console.log("  - validUntil:", validUntil);
        console.log("  - validAfter:", validAfter);
        console.log("  - paymasterVerificationGasLimit:", paymasterVerificationGasLimit);
        console.log("  - paymasterPostOpGasLimit:", paymasterPostOpGasLimit);
        
        // Create paymasterAndData with SimplePaymaster format
        userOp.paymasterAndData = abi.encodePacked(
            address(paymaster), 
            paymasterVerificationGasLimit, 
            paymasterPostOpGasLimit,
            validUntil,
            validAfter
        );
        
        console.log("  - paymasterAndData length:", userOp.paymasterAndData.length);
        
        return userOp;
    }
    
    function signUserOperation(PackedUserOperation memory userOp, bytes32 userOpHash) 
        internal view returns (PackedUserOperation memory) {
        
        console.log("  - signing with userOpHash:", vm.toString(userOpHash));
        
        // Sign with account owner key
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(accountOwnerKey, userOpHash);
        userOp.signature = abi.encodePacked(r, s, v);
        
        return userOp;
    }
}
