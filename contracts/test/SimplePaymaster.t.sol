// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "forge-std/Test.sol";
import "../src/SimplePaymaster.sol";
import "../lib/account-abstraction/contracts/core/EntryPoint.sol";
import "../lib/account-abstraction/contracts/samples/SimpleAccount.sol";
import "../lib/account-abstraction/contracts/samples/SimpleAccountFactory.sol";
import "../lib/account-abstraction/contracts/interfaces/PackedUserOperation.sol";
import "../lib/account-abstraction/contracts/core/UserOperationLib.sol";
import "../lib/account-abstraction/contracts/core/Helpers.sol";

contract SimplePaymasterTest is Test {
    SimplePaymaster public paymaster;
    EntryPoint public entryPoint;
    SimpleAccountFactory public factory;
    SimpleAccount public account;
    
    address public owner = address(0x1234);
    address public user = address(0x5678);
    address public beneficiary = address(0x9abc);
    
    uint256 public constant MAX_GAS_COST = 0.01 ether;
    
    event GasSponsored(address indexed user, uint256 actualGasCost, bytes32 userOpHash);
    event MaxGasCostUpdated(uint256 oldValue, uint256 newValue);
    event PaymasterToggled(bool isActive);

    function setUp() public {
        // Deploy EntryPoint
        entryPoint = new EntryPoint();
        
        // Deploy SimplePaymaster
        paymaster = new SimplePaymaster(IEntryPoint(address(entryPoint)), MAX_GAS_COST);
        
        // Deploy SimpleAccountFactory
        factory = new SimpleAccountFactory(IEntryPoint(address(entryPoint)));
        
        // Create a SimpleAccount
        account = SimpleAccount(payable(factory.createAccount(owner, 0)));
        
        // Fund the paymaster
        vm.deal(address(paymaster), 10 ether);
        paymaster.deposit{value: 10 ether}();
        
        // Fund the account
        vm.deal(address(account), 1 ether);
        
        // Fund the owner
        vm.deal(owner, 1 ether);
    }

    function testConstructor() public {
        assertEq(address(paymaster.entryPoint()), address(entryPoint));
        assertEq(paymaster.maxGasCost(), MAX_GAS_COST);
        assertTrue(paymaster.isActive());
        assertEq(paymaster.owner(), address(this));
    }

    function testConstructorRevertsOnZeroEntryPoint() public {
        vm.expectRevert("EntryPoint cannot be zero");
        new SimplePaymaster(IEntryPoint(address(0)), MAX_GAS_COST);
    }

    function testConstructorRevertsOnZeroMaxGasCost() public {
        vm.expectRevert("Max gas cost must be positive");
        new SimplePaymaster(IEntryPoint(address(entryPoint)), 0);
    }

    function testValidatePaymasterUserOp() public {
        // Create a valid UserOperation
        PackedUserOperation memory userOp = _createValidUserOp();
        
        // Mock EntryPoint as caller
        vm.prank(address(entryPoint));
        
        (bytes memory context, uint256 validationData) = paymaster.validatePaymasterUserOp(
            userOp,
            keccak256("userOpHash"),
            MAX_GAS_COST
        );
        
        // Verify context contains userOpHash and sender
        (bytes32 userOpHash, address sender) = abi.decode(context, (bytes32, address));
        assertEq(sender, user);
        
        // Verify validation data (no time restrictions)
        ValidationData memory data = _parseValidationData(validationData);
        assertEq(data.aggregator, address(0));
        assertEq(data.validAfter, 0);
        assertEq(data.validUntil, type(uint48).max);
    }

    function testValidatePaymasterUserOpRevertsOnNonEntryPoint() public {
        PackedUserOperation memory userOp = _createValidUserOp();
        
        vm.expectRevert(SimplePaymaster.OnlyEntryPoint.selector);
        paymaster.validatePaymasterUserOp(
            userOp,
            keccak256("userOpHash"),
            MAX_GAS_COST
        );
    }

    function testValidatePaymasterUserOpRevertsWhenInactive() public {
        // Deactivate paymaster
        paymaster.togglePaymaster();
        assertFalse(paymaster.isActive());
        
        PackedUserOperation memory userOp = _createValidUserOp();
        
        vm.prank(address(entryPoint));
        vm.expectRevert(SimplePaymaster.PaymasterInactive.selector);
        paymaster.validatePaymasterUserOp(
            userOp,
            keccak256("userOpHash"),
            MAX_GAS_COST
        );
    }

    function testValidatePaymasterUserOpRevertsOnHighGasCost() public {
        PackedUserOperation memory userOp = _createValidUserOp();
        
        vm.prank(address(entryPoint));
        vm.expectRevert(abi.encodeWithSelector(
            SimplePaymaster.GasCostTooHigh.selector,
            MAX_GAS_COST,
            MAX_GAS_COST + 1
        ));
        paymaster.validatePaymasterUserOp(
            userOp,
            keccak256("userOpHash"),
            MAX_GAS_COST + 1
        );
    }

    function testValidatePaymasterUserOpRevertsOnWrongPaymaster() public {
        PackedUserOperation memory userOp = _createValidUserOp();
        
        // Modify paymasterAndData to point to wrong paymaster
        bytes memory wrongPaymasterData = abi.encodePacked(
            address(0xdead), // Wrong paymaster address
            uint128(100_000), // validationGas
            uint128(50_000),  // postOpGas
            bytes("extra data")
        );
        userOp.paymasterAndData = wrongPaymasterData;
        
        vm.prank(address(entryPoint));
        vm.expectRevert("Wrong paymaster address");
        paymaster.validatePaymasterUserOp(
            userOp,
            keccak256("userOpHash"),
            MAX_GAS_COST
        );
    }

    function testValidatePaymasterUserOpRevertsOnLowGasLimits() public {
        PackedUserOperation memory userOp = _createValidUserOp();
        
        // Modify paymasterAndData with low gas limits
        bytes memory lowGasData = abi.encodePacked(
            address(paymaster),
            uint128(30_000), // Too low validation gas
            uint128(50_000), // postOpGas
            bytes("extra data")
        );
        userOp.paymasterAndData = lowGasData;
        
        vm.prank(address(entryPoint));
        vm.expectRevert("Validation gas too low");
        paymaster.validatePaymasterUserOp(
            userOp,
            keccak256("userOpHash"),
            MAX_GAS_COST
        );
    }

    function testPostOp() public {
        bytes memory context = abi.encode(keccak256("userOpHash"), user);
        
        vm.prank(address(entryPoint));
        
        // Expect event emission
        vm.expectEmit(true, true, false, true);
        emit GasSponsored(user, 0.001 ether, keccak256("userOpHash"));
        
        paymaster.postOp(
            IPaymaster.PostOpMode.opSucceeded,
            context,
            0.001 ether,
            20 gwei
        );
    }

    function testPostOpRevertsOnNonEntryPoint() public {
        bytes memory context = abi.encode(keccak256("userOpHash"), user);
        
        vm.expectRevert(SimplePaymaster.OnlyEntryPoint.selector);
        paymaster.postOp(
            IPaymaster.PostOpMode.opSucceeded,
            context,
            0.001 ether,
            20 gwei
        );
    }

    function testSetMaxGasCost() public {
        uint256 newMaxGasCost = 0.02 ether;
        
        vm.expectEmit(true, false, false, true);
        emit MaxGasCostUpdated(MAX_GAS_COST, newMaxGasCost);
        
        paymaster.setMaxGasCost(newMaxGasCost);
        assertEq(paymaster.maxGasCost(), newMaxGasCost);
    }

    function testSetMaxGasCostRevertsOnZero() public {
        vm.expectRevert("Max gas cost must be positive");
        paymaster.setMaxGasCost(0);
    }

    function testSetMaxGasCostRevertsOnNonOwner() public {
        vm.prank(user);
        vm.expectRevert();
        paymaster.setMaxGasCost(0.02 ether);
    }

    function testTogglePaymaster() public {
        assertTrue(paymaster.isActive());
        
        vm.expectEmit(true, false, false, true);
        emit PaymasterToggled(false);
        
        paymaster.togglePaymaster();
        assertFalse(paymaster.isActive());
        
        vm.expectEmit(true, false, false, true);
        emit PaymasterToggled(true);
        
        paymaster.togglePaymaster();
        assertTrue(paymaster.isActive());
    }

    function testTogglePaymasterRevertsOnNonOwner() public {
        vm.prank(user);
        vm.expectRevert();
        paymaster.togglePaymaster();
    }

    function testDeposit() public {
        uint256 initialDeposit = entryPoint.balanceOf(address(paymaster));
        
        vm.deal(address(this), 1 ether);
        paymaster.deposit{value: 1 ether}();
        
        uint256 finalDeposit = entryPoint.balanceOf(address(paymaster));
        assertEq(finalDeposit, initialDeposit + 1 ether);
    }

    function testWithdrawTo() public {
        uint256 withdrawAmount = 0.5 ether;
        uint256 initialBalance = beneficiary.balance;
        
        paymaster.withdrawTo(payable(beneficiary), withdrawAmount);
        
        uint256 finalBalance = beneficiary.balance;
        assertEq(finalBalance, initialBalance + withdrawAmount);
    }

    function testWithdrawToRevertsOnNonOwner() public {
        vm.prank(user);
        vm.expectRevert();
        paymaster.withdrawTo(payable(beneficiary), 0.1 ether);
    }

    function testAddStake() public {
        uint32 unstakeDelay = 86400; // 1 day
        
        vm.deal(address(this), 1 ether);
        paymaster.addStake{value: 1 ether}(unstakeDelay);
        
        // Verify stake was added (this would require additional EntryPoint methods to verify fully)
        // For now, we just ensure the call doesn't revert
    }

    function testGetDeposit() public {
        uint256 deposit = paymaster.getDeposit();
        assertEq(deposit, entryPoint.balanceOf(address(paymaster)));
    }

    function testUnlockStake() public {
        paymaster.unlockStake();
        // This would require additional EntryPoint methods to verify fully
        // For now, we just ensure the call doesn't revert
    }

    function testWithdrawStake() public {
        paymaster.withdrawStake(payable(beneficiary));
        // This would require additional EntryPoint methods to verify fully
        // For now, we just ensure the call doesn't revert
    }

    function testReceive() public {
        uint256 initialDeposit = entryPoint.balanceOf(address(paymaster));
        
        vm.deal(address(this), 1 ether);
        (bool success,) = address(paymaster).call{value: 1 ether}("");
        assertTrue(success);
        
        uint256 finalDeposit = entryPoint.balanceOf(address(paymaster));
        assertEq(finalDeposit, initialDeposit + 1 ether);
    }

    function testEmergencyWithdraw() public {
        // Send some ETH directly to the contract
        vm.deal(address(paymaster), 1 ether);
        
        uint256 initialBalance = beneficiary.balance;
        paymaster.emergencyWithdraw(payable(beneficiary));
        
        uint256 finalBalance = beneficiary.balance;
        assertEq(finalBalance, initialBalance + 1 ether);
        assertEq(address(paymaster).balance, 0);
    }

    function testEmergencyWithdrawRevertsOnNonOwner() public {
        vm.prank(user);
        vm.expectRevert();
        paymaster.emergencyWithdraw(payable(beneficiary));
    }

    function testEmergencyWithdrawRevertsOnZeroBalance() public {
        vm.expectRevert("No ETH to withdraw");
        paymaster.emergencyWithdraw(payable(beneficiary));
    }

    // Helper function to create a valid UserOperation
    function _createValidUserOp() internal view returns (PackedUserOperation memory) {
        bytes memory paymasterData = abi.encodePacked(
            address(paymaster),      // paymaster address
            uint128(100_000),       // validationGas
            uint128(50_000),        // postOpGas
            bytes("extra data")     // custom data
        );
        
        return PackedUserOperation({
            sender: user,
            nonce: 0,
            initCode: "",
            callData: "",
            accountGasLimits: 0,
            preVerificationGas: 0,
            gasFees: 0,
            paymasterAndData: paymasterData,
            signature: ""
        });
    }
}
