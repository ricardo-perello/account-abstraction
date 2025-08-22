// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";
import "@openzeppelin/contracts/utils/cryptography/MessageHashUtils.sol";
import "@openzeppelin/contracts/utils/structs/EnumerableSet.sol";
import "@account-abstraction/contracts/core/BaseAccount.sol";
import "@account-abstraction/contracts/core/Helpers.sol";

contract AAAccount is BaseAccount {
    using ECDSA for bytes32;
    using EnumerableSet for EnumerableSet.AddressSet;

    // Owner management using EnumerableSet for gas efficiency
    mapping(address => bool) public owners;
    EnumerableSet.AddressSet private _ownerSet;
    

    
    // Events
    event OwnerAdded(address indexed owner);
    event OwnerRemoved(address indexed owner);
    event TransactionExecuted(address indexed target, uint256 value, bytes data);
    event BatchTransactionExecuted(address[] targets, uint256[] values, bytes[] datas);
    event AccountInitialized(address indexed owner);

    IEntryPoint private immutable _entryPoint;
    bool private _initialized;

    constructor(IEntryPoint anEntryPoint, address initialOwner) {
        _entryPoint = anEntryPoint;
        
        // Only initialize if initialOwner is not zero (direct deployment)
        if (initialOwner != address(0)) {
            _initialize(initialOwner);
        }
    }

    /**
     * @dev Initialize the account with a single owner (for factory deployment)
     * @param owner The initial owner
     */
    function initialize(address owner) external {
        require(!_initialized, "AAAccount: already initialized");
        require(owner != address(0), "AAAccount: owner cannot be zero");
        _initialized = true;
        _initialize(owner);
    }

    /**
     * @dev Initialize the account with multiple owners (for factory deployment)
     * @param initialOwners Array of initial owners
     */
    function initializeWithOwners(address[] calldata initialOwners) external {
        require(!_initialized, "AAAccount: already initialized");
        require(initialOwners.length > 0, "AAAccount: owners array cannot be empty");
        require(initialOwners.length <= 10, "AAAccount: too many owners (max 10)");
        
        _initialized = true;
        
        // Validate and add all owners
        for (uint256 i = 0; i < initialOwners.length; i++) {
            require(initialOwners[i] != address(0), "AAAccount: owner cannot be zero");
            require(!owners[initialOwners[i]], "AAAccount: duplicate owner");
            
            owners[initialOwners[i]] = true;
            _ownerSet.add(initialOwners[i]);
            emit OwnerAdded(initialOwners[i]);
        }
        emit AccountInitialized(initialOwners[0]);
    }

    /**
     * @dev Internal initialization function
     * @param owner The initial owner
     */
    function _initialize(address owner) private {
        owners[owner] = true;
        _ownerSet.add(owner);
        emit OwnerAdded(owner);
        emit AccountInitialized(owner);
    }

    function entryPoint() public view override returns (IEntryPoint) {
        return _entryPoint;
    }

    // Owner management functions
    function addOwner(address newOwner) external {
        require(owners[msg.sender], "AAAccount: caller is not an owner");
        require(newOwner != address(0), "AAAccount: invalid owner address");
        require(!owners[newOwner], "AAAccount: owner already exists");
        
        owners[newOwner] = true;
        _ownerSet.add(newOwner);
        
        emit OwnerAdded(newOwner);
    }
    
    function removeOwner(address ownerToRemove) external {
        require(owners[msg.sender], "AAAccount: caller is not an owner");
        require(ownerToRemove != msg.sender, "AAAccount: cannot remove self");
        require(owners[ownerToRemove], "AAAccount: owner does not exist");
        require(_ownerSet.length() > 1, "AAAccount: cannot remove last owner");
        
        owners[ownerToRemove] = false;
        _ownerSet.remove(ownerToRemove);
        
        emit OwnerRemoved(ownerToRemove);
    }
    
    function getOwners() external view returns (address[] memory) {
        return _ownerSet.values();
    }
    
    function isOwner(address account) external view returns (bool) {
        return owners[account];
    }
    
    function ownerCount() external view returns (uint256) {
        return _ownerSet.length();
    }
    
    function ownerList(uint256 index) external view returns (address) {
        return _ownerSet.at(index);
    }

    function _requireForExecute() internal view override {
        require(msg.sender == address(entryPoint()), "account: not EntryPoint");
    }

    function _validateSignature(
        PackedUserOperation calldata userOp,
        bytes32 userOpHash
    ) internal override returns (uint256 validationData) {
        // userOpHash is already the EIP-712 typed data hash, use it directly
        // No need to apply EIP-191 formatting again
        
        // Use ECDSA.tryRecover for safe signature recovery
        (address signer, ECDSA.RecoverError error,) = ECDSA.tryRecover(userOpHash, userOp.signature);
        
        // Check for recovery errors
        if (error != ECDSA.RecoverError.NoError) {
            return SIG_VALIDATION_FAILED;
        }
        
        // Check if signer is an authorized owner
        if (!owners[signer]) {
            return SIG_VALIDATION_FAILED;
        }
        
        return SIG_VALIDATION_SUCCESS;
    }

    // Let EntryPoint handle nonce validation - no custom override needed

    /**
     * @dev EIP-1271 signature validation
     * @param hash The hash that was signed
     * @param signature The signature to validate
     * @return magicValue EIP-1271 magic value if valid, 0 otherwise
     */
    function isValidSignature(bytes32 hash, bytes calldata signature) external view returns (bytes4 magicValue) {
        // Use ECDSA.tryRecover for safe signature recovery
        (address signer, ECDSA.RecoverError error,) = ECDSA.tryRecover(hash, signature);
        
        // Check for recovery errors or if signer is not an owner
        if (error != ECDSA.RecoverError.NoError || !owners[signer]) {
            return 0x00000000; // Invalid signature
        }
        
        // Return EIP-1271 magic value for valid signature
        return 0x1626ba7e; // bytes4(keccak256("isValidSignature(bytes32,bytes)"))
    }

    // Custom execution functions that can be called by owners or EntryPoint
    function execute(address target, uint256 value, bytes calldata data) external override {
        _requireForExecute();
        require(target != address(0), "AAAccount: invalid target");
        
        (bool success, ) = target.call{value: value}(data);
        require(success, "AAAccount: execution failed");
        
        emit TransactionExecuted(target, value, data);
    }

    /**
     * @dev Execute a transaction directly from an owner (bypassing EntryPoint)
     * @param target The target contract address
     * @param value The ETH value to send
     * @param data The call data
     */
    function executeDirectly(address target, uint256 value, bytes calldata data) external {
        require(owners[msg.sender], "AAAccount: caller is not an owner");
        require(target != address(0), "AAAccount: invalid target");
        
        (bool success, ) = target.call{value: value}(data);
        require(success, "AAAccount: execution failed");
        
        emit TransactionExecuted(target, value, data);
    }

    function executeBatch(address[] calldata targets, uint256[] calldata values, bytes[] calldata datas) external {
        _requireForExecute();
        require(
            targets.length == values.length && targets.length == datas.length,
            "AAAccount: array length mismatch"
        );
        
        for (uint256 i = 0; i < targets.length; i++) {
            require(targets[i] != address(0), "AAAccount: invalid target");
            
            (bool success, ) = targets[i].call{value: values[i]}(datas[i]);
            require(success, "AAAccount: batch execution failed");
        }
        
        emit BatchTransactionExecuted(targets, values, datas);
    }

    /**
     * @dev Execute multiple transactions directly from an owner (bypassing EntryPoint)
     * @param targets Array of target contract addresses
     * @param values Array of ETH values to send
     * @param datas Array of call data
     */
    function executeBatchDirectly(address[] calldata targets, uint256[] calldata values, bytes[] calldata datas) external {
        require(owners[msg.sender], "AAAccount: caller is not an owner");
        require(
            targets.length == values.length && targets.length == datas.length,
            "AAAccount: array length mismatch"
        );
        
        for (uint256 i = 0; i < targets.length; i++) {
            require(targets[i] != address(0), "AAAccount: invalid target");
            
            (bool success, ) = targets[i].call{value: values[i]}(datas[i]);
            require(success, "AAAccount: batch execution failed");
        }
        
        emit BatchTransactionExecuted(targets, values, datas);
    }

    receive() external payable {}
}
