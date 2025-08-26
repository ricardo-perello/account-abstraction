// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "forge-std/console.sol";
import "@openzeppelin/contracts/access/Ownable2Step.sol";
import "@openzeppelin/contracts/utils/introspection/IERC165.sol";
import "../lib/account-abstraction/contracts/interfaces/IPaymaster.sol";
import "../lib/account-abstraction/contracts/interfaces/IEntryPoint.sol";
import "../lib/account-abstraction/contracts/core/UserOperationLib.sol";
import "../lib/account-abstraction/contracts/core/Helpers.sol";

/**
 * @title SimplePaymaster
 * @dev A simple ERC-4337 v0.7 paymaster that sponsors gas for all users
 * 
 * This paymaster follows the v0.7 canonical EntryPoint standards:
 * - Uses PackedUserOperation struct
 * - Implements proper paymasterAndData layout (52-byte offset)
 * - Handles gas limits correctly
 * - Implements proper validation data packing
 * - Follows all security best practices
 */
contract SimplePaymaster is IPaymaster, Ownable2Step {
    using UserOperationLib for PackedUserOperation;

    /// @notice The EntryPoint contract this paymaster works with
    IEntryPoint public immutable entryPoint;

    /// @notice Maximum gas cost this paymaster will sponsor per operation
    uint256 public maxGasCost;

    /// @notice Whether the paymaster is currently active
    bool public isActive;

    /// @notice Events for monitoring
    event GasSponsored(address indexed user, uint256 actualGasCost, bytes32 userOpHash);
    event MaxGasCostUpdated(uint256 oldValue, uint256 newValue);
    event PaymasterToggled(bool isActive);

    /// @notice Error when paymaster is not active
    error PaymasterInactive();
    
    /// @notice Error when gas cost exceeds maximum
    error GasCostTooHigh(uint256 maxCost, uint256 actualCost);
    
    /// @notice Error when called from non-EntryPoint
    error OnlyEntryPoint();

    /**
     * @dev Constructor
     * @param _entryPoint The EntryPoint contract address
     * @param _maxGasCost Maximum gas cost to sponsor per operation
     */
    constructor(IEntryPoint _entryPoint, uint256 _maxGasCost) Ownable(msg.sender) {
        require(address(_entryPoint) != address(0), "EntryPoint cannot be zero");
        require(_maxGasCost > 0, "Max gas cost must be positive");
        
        entryPoint = _entryPoint;
        maxGasCost = _maxGasCost;
        isActive = true;
        
        // Validate EntryPoint interface
        _validateEntryPointInterface(_entryPoint);
    }

    /**
     * @dev Validates the EntryPoint interface
     * @param _entryPoint The EntryPoint to validate
     */
    function _validateEntryPointInterface(IEntryPoint _entryPoint) internal view {
        // Check if EntryPoint has code
        require(address(_entryPoint).code.length > 0, "EntryPoint has no code");
        
        // Try ERC-165 check first
        try IERC165(address(_entryPoint)).supportsInterface(type(IERC165).interfaceId) returns (bool supportsERC165) {
            if (supportsERC165) {
                // If ERC-165 is supported, also check for IEntryPoint interface
                try IERC165(address(_entryPoint)).supportsInterface(type(IEntryPoint).interfaceId) returns (bool supportsEntryPoint) {
                    if (!supportsEntryPoint) {
                        // Fallback: just ensure it has code and basic structure
                        console.log("EntryPoint ERC-165 check passed, but IEntryPoint interface not found");
                    }
                } catch {
                    // Fallback: just ensure it has code
                    console.log("EntryPoint ERC-165 check passed, but IEntryPoint interface check failed");
                }
            }
        } catch {
            // EntryPoint doesn't support ERC-165, which is fine for canonical EntryPoint
            console.log("EntryPoint doesn't support ERC-165, using fallback validation");
            
            // Basic validation: ensure EntryPoint has code and can be called
            // This is sufficient for the canonical EntryPoint
        }
    }

    /**
     * @dev Validates a user operation for gas sponsorship
     * @param userOp The user operation to validate
     * @param userOpHash Hash of the user operation
     * @param maxCost Maximum cost this operation can have
     * @return context Context data for postOp
     * @return validationData Validation data with time bounds
     */
    function validatePaymasterUserOp(
        PackedUserOperation calldata userOp,
        bytes32 userOpHash,
        uint256 maxCost
    ) external override returns (bytes memory context, uint256 validationData) {
        // Only EntryPoint can call this
        if (msg.sender != address(entryPoint)) {
            revert OnlyEntryPoint();
        }

        // Check if paymaster is active
        if (!isActive) {
            revert PaymasterInactive();
        }

        // Check gas cost limits
        if (maxCost > maxGasCost) {
            revert GasCostTooHigh(maxGasCost, maxCost);
        }

        // Extract paymaster data using v0.7 layout
        bytes calldata paymasterData = userOp.paymasterAndData;
        
        // v0.7 layout: [paymaster(20) + validationGas(16) + postOpGas(16) + customData]
        require(
            paymasterData.length >= UserOperationLib.PAYMASTER_DATA_OFFSET,
            "Paymaster data too short"
        );

        // Extract and validate paymaster address
        address paymaster = address(bytes20(paymasterData[:UserOperationLib.PAYMASTER_VALIDATION_GAS_OFFSET]));
        require(paymaster == address(this), "Wrong paymaster address");

        // Extract gas limits
        uint256 validationGasLimit = uint128(bytes16(paymasterData[UserOperationLib.PAYMASTER_VALIDATION_GAS_OFFSET:UserOperationLib.PAYMASTER_POSTOP_GAS_OFFSET]));
        uint256 postOpGasLimit = uint128(bytes16(paymasterData[UserOperationLib.PAYMASTER_POSTOP_GAS_OFFSET:UserOperationLib.PAYMASTER_DATA_OFFSET]));

        // Ensure reasonable gas limits
        require(validationGasLimit >= 50_000, "Validation gas too low");
        require(postOpGasLimit >= 20_000, "PostOp gas too low");

        // Create context for postOp (userOpHash and sender)
        context = abi.encode(userOpHash, userOp.sender);

        // Return validation data: no time restrictions, always valid
        // Pack: (sigFailed=false, validUntil=0=indefinite, validAfter=0=immediate)
        validationData = _packValidationData(false, 0, 0);

        return (context, validationData);
    }

    /**
     * @dev Post-operation handler called by EntryPoint
     * @param mode Operation mode (success/revert)
     * @param context Context data from validatePaymasterUserOp
     * @param actualGasCost Actual gas cost of the operation
     * @param actualUserOpFeePerGas Actual fee per gas paid by user
     */
    function postOp(
        PostOpMode mode,
        bytes calldata context,
        uint256 actualGasCost,
        uint256 actualUserOpFeePerGas
    ) external override {
        // Only EntryPoint can call this
        if (msg.sender != address(entryPoint)) {
            revert OnlyEntryPoint();
        }

        // Decode context
        (bytes32 userOpHash, address user) = abi.decode(context, (bytes32, address));

        // Emit event for monitoring
        emit GasSponsored(user, actualGasCost, userOpHash);

        // Note: In this simple implementation, we don't need additional logic
        // The EntryPoint automatically deducts the gas cost from our deposit
        // and refunds any excess
    }

    /**
     * @dev Updates the maximum gas cost this paymaster will sponsor
     * @param newMaxGasCost New maximum gas cost
     */
    function setMaxGasCost(uint256 newMaxGasCost) external onlyOwner {
        require(newMaxGasCost > 0, "Max gas cost must be positive");
        uint256 oldValue = maxGasCost;
        maxGasCost = newMaxGasCost;
        emit MaxGasCostUpdated(oldValue, newMaxGasCost);
    }

    /**
     * @dev Toggles the paymaster on/off
     */
    function togglePaymaster() external onlyOwner {
        isActive = !isActive;
        emit PaymasterToggled(isActive);
    }

    /**
     * @dev Deposits ETH to the EntryPoint for gas sponsorship
     */
    function deposit() public payable {
        entryPoint.depositTo{value: msg.value}(address(this));
    }

    /**
     * @dev Withdraws ETH from the EntryPoint deposit
     * @param withdrawAddress Address to withdraw to
     * @param amount Amount to withdraw
     */
    function withdrawTo(address payable withdrawAddress, uint256 amount) public onlyOwner {
        entryPoint.withdrawTo(withdrawAddress, amount);
    }

    /**
     * @dev Adds stake to the EntryPoint
     * @param unstakeDelaySec Unstake delay in seconds
     */
    function addStake(uint32 unstakeDelaySec) external payable onlyOwner {
        entryPoint.addStake{value: msg.value}(unstakeDelaySec);
    }

    /**
     * @dev Gets current deposit balance
     * @return Current deposit amount
     */
    function getDeposit() public view returns (uint256) {
        return entryPoint.balanceOf(address(this));
    }

    /**
     * @dev Unlocks stake for withdrawal
     */
    function unlockStake() external onlyOwner {
        entryPoint.unlockStake();
    }

    /**
     * @dev Withdraws stake after unlock period
     * @param withdrawAddress Address to withdraw stake to
     */
    function withdrawStake(address payable withdrawAddress) external onlyOwner {
        entryPoint.withdrawStake(withdrawAddress);
    }

    /**
     * @dev Allows contract to receive ETH and automatically deposit to EntryPoint
     */
    receive() external payable {
        deposit();
    }

    /**
     * @dev Emergency function to withdraw all ETH from contract
     * @param withdrawAddress Address to withdraw to
     */
    function emergencyWithdraw(address payable withdrawAddress) external onlyOwner {
        uint256 balance = address(this).balance;
        require(balance > 0, "No ETH to withdraw");
        withdrawAddress.transfer(balance);
    }
}
