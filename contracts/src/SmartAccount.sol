// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./interfaces/IEntryPoint.sol";
import "./interfaces/UserOperation.sol";

contract SmartAccount {
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
    
    // Modifiers
    modifier onlyOwner() {
        require(owners[msg.sender], "SmartAccount: caller is not an owner");
        _;
    }
    
    modifier onlyEntryPoint() {
        require(msg.sender == address(entryPoint), "SmartAccount: caller is not entry point");
        _;
    }
    
    modifier onlySelf() {
        require(msg.sender == address(this), "SmartAccount: caller is not self");
        _;
    }
    
    // Entry point reference
    IEntryPoint public immutable entryPoint;
    
    constructor(address _entryPoint) {
        require(_entryPoint != address(0), "SmartAccount: invalid entry point");
        entryPoint = IEntryPoint(_entryPoint);
        
        // Add deployer as first owner
        owners[msg.sender] = true;
        ownerList.push(msg.sender);
        ownerCount = 1;
        
        emit OwnerAdded(msg.sender);
    }
    
    // Owner management functions
    function addOwner(address newOwner) external onlyOwner {
        require(newOwner != address(0), "SmartAccount: invalid owner address");
        require(!owners[newOwner], "SmartAccount: owner already exists");
        
        owners[newOwner] = true;
        ownerList.push(newOwner);
        ownerCount++;
        
        emit OwnerAdded(newOwner);
    }
    
    function removeOwner(address ownerToRemove) external onlyOwner {
        require(ownerToRemove != msg.sender, "SmartAccount: cannot remove self");
        require(owners[ownerToRemove], "SmartAccount: owner does not exist");
        require(ownerCount > 1, "SmartAccount: cannot remove last owner");
        
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
    
    // Core ERC-4337 functions
    function validateUserOp(UserOperation calldata userOp, bytes32 userOpHash, uint256 missingAccountFunds)
        external
        onlyEntryPoint
        returns (uint256)
    {
        // Verify nonce hasn't been used
        require(nonces[uint192(userOp.nonce)] == 0, "SmartAccount: nonce already used");
        
        // Mark nonce as used
        nonces[uint192(userOp.nonce)] = 1;
        
        // Verify signature
        require(isValidSignature(userOpHash, userOp.signature), "SmartAccount: invalid signature");
        
        // Return validation gas cost (0 for now, can be adjusted based on complexity)
        return 0;
    }

    function execute(address target, uint256 value, bytes calldata data) external onlySelf {
        require(target != address(0), "SmartAccount: invalid target");
        
        (bool success, ) = target.call{value: value}(data);
        require(success, "SmartAccount: execution failed");
        
        emit TransactionExecuted(target, value, data);
    }

    function executeBatch(address[] calldata targets, uint256[] calldata values, bytes[] calldata datas)
        external
        onlySelf
    {
        require(
            targets.length == values.length && targets.length == datas.length,
            "SmartAccount: array length mismatch"
        );
        
        for (uint256 i = 0; i < targets.length; i++) {
            require(targets[i] != address(0), "SmartAccount: invalid target");
            
            (bool success, ) = targets[i].call{value: values[i]}(datas[i]);
            require(success, "SmartAccount: batch execution failed");
        }
        
        emit BatchTransactionExecuted(targets, values, datas);
    }

    function getNonce(uint192 key) external view returns (uint256) {
        return nonces[key];
    }

    function isValidSignature(bytes32 hash, bytes calldata signature) public view returns (bool) {
        // Check if signature length is valid (65 bytes: r, s, v)
        if (signature.length != 65) {
            return false;
        }
        
        // Extract r, s, v from signature using memory
        bytes memory sig = signature;
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
            return false;
        }
        
        // Recover signer address
        address signer = ecrecover(hash, v, r, s);
        
        // Check if signer is an authorized owner
        return owners[signer];
    }
    
    // Function to execute transactions from EntryPoint
    function _executeFromEntryPoint(address target, uint256 value, bytes calldata data) internal {
        require(target != address(0), "SmartAccount: invalid target");
        
        (bool success, ) = target.call{value: value}(data);
        require(success, "SmartAccount: execution failed");
        
        emit TransactionExecuted(target, value, data);
    }
    
    function _executeBatchFromEntryPoint(address[] calldata targets, uint256[] calldata values, bytes[] calldata datas) internal {
        require(
            targets.length == values.length && targets.length == datas.length,
            "SmartAccount: array length mismatch"
        );
        
        for (uint256 i = 0; i < targets.length; i++) {
            require(targets[i] != address(0), "SmartAccount: invalid target");
            
            (bool success, ) = targets[i].call{value: values[i]}(datas[i]);
            require(success, "SmartAccount: batch execution failed");
        }
        
        emit BatchTransactionExecuted(targets, values, datas);
    }

    receive() external payable {}
}
