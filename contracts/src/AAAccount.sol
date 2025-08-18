// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";
import "@account-abstraction/contracts/core/BaseAccount.sol";
import "@account-abstraction/contracts/core/Helpers.sol";

contract AAAccount is BaseAccount {
    using ECDSA for bytes32;

    // Owner management
    mapping(address => bool) public owners;
    address[] public ownerList;
    uint256 public ownerCount;
    
    // Nonce management for replay protection
    mapping(uint192 => uint256) public nonces;
    
    // Events
    event OwnerAdded(address indexed owner);
    event OwnerRemoved(address indexed owner);
    event TransactionExecuted(address indexed target, uint256 value, bytes data);
    event BatchTransactionExecuted(address[] targets, uint256[] values, bytes[] datas);

    IEntryPoint private immutable _entryPoint;

    constructor(IEntryPoint anEntryPoint, address initialOwner) {
        _entryPoint = anEntryPoint;
        
        // Add initial owner
        owners[initialOwner] = true;
        ownerList.push(initialOwner);
        ownerCount = 1;
        
        emit OwnerAdded(initialOwner);
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
        ownerList.push(newOwner);
        ownerCount++;
        
        emit OwnerAdded(newOwner);
    }
    
    function removeOwner(address ownerToRemove) external {
        require(owners[msg.sender], "AAAccount: caller is not an owner");
        require(ownerToRemove != msg.sender, "AAAccount: cannot remove self");
        require(owners[ownerToRemove], "AAAccount: owner does not exist");
        require(ownerCount > 1, "AAAccount: cannot remove last owner");
        
        owners[ownerToRemove] = false;
        ownerCount--;
        
        // Remove from ownerList array
        for (uint256 i = 0; i < ownerList.length; i++) {
            if (ownerList[i] == ownerToRemove) {
                ownerList[i] = ownerList[ownerList.length - 1];
                ownerList.pop();
                break;
            }
        }
        
        emit OwnerRemoved(ownerToRemove);
    }
    
    function getOwners() external view returns (address[] memory) {
        address[] memory activeOwners = new address[](ownerCount);
        uint256 activeIndex = 0;
        
        for (uint256 i = 0; i < ownerList.length; i++) {
            if (owners[ownerList[i]]) {
                activeOwners[activeIndex] = ownerList[i];
                activeIndex++;
            }
        }
        
        return activeOwners;
    }
    
    function isOwner(address account) external view returns (bool) {
        return owners[account];
    }

    function _requireForExecute() internal view override {
        require(
            msg.sender == address(entryPoint()) || owners[msg.sender],
            "account: not Owner or EntryPoint"
        );
    }

    function _validateSignature(
        PackedUserOperation calldata userOp,
        bytes32 userOpHash
    ) internal override returns (uint256 validationData) {
        // Check if signature length is valid (65 bytes: r, s, v)
        if (userOp.signature.length != 65) {
            return SIG_VALIDATION_FAILED;
        }
        
        // Extract r, s, v from signature using memory
        bytes memory sig = userOp.signature;
        bytes32 r;
        bytes32 s;
        uint8 v;
        
        assembly {
            r := mload(add(sig, 32))
            s := mload(add(sig, 64))
            v := byte(0, mload(add(sig, 96)))
        }
        
        // Ensure v is valid (27 or 28)
        if (v < 27) v += 27;
        if (v != 27 && v != 28) {
            return SIG_VALIDATION_FAILED;
        }
        
        // Recover signer address
        address signer = ecrecover(userOpHash, v, r, s);
        
        // Check if signer is an authorized owner
        if (!owners[signer]) {
            return SIG_VALIDATION_FAILED;
        }
        
        return SIG_VALIDATION_SUCCESS;
    }

    // Override _validateNonce to implement custom nonce validation
    function _validateNonce(uint256 nonce) internal view override {
        // Verify nonce hasn't been used
        require(nonces[uint192(nonce)] == 0, "AAAccount: nonce already used");
    }

    // Override validateUserOp to mark nonce as used after validation
    function validateUserOp(
        PackedUserOperation calldata userOp,
        bytes32 userOpHash,
        uint256 missingAccountFunds
    ) external override returns (uint256 validationData) {
        _requireFromEntryPoint();
        
        // Validate signature and nonce first
        validationData = _validateSignature(userOp, userOpHash);
        _validateNonce(userOp.nonce);
        
        // Mark nonce as used after successful validation
        nonces[uint192(userOp.nonce)] = 1;
        
        _payPrefund(missingAccountFunds);
    }

    // Nonce getter for external access
    function getNonce(uint192 key) external view returns (uint256) {
        return nonces[key];
    }

    // Custom execution functions that can be called by owners or EntryPoint
    function execute(address target, uint256 value, bytes calldata data) external override {
        _requireForExecute();
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

    receive() external payable {}
}
