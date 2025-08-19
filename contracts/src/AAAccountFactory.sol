// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "./AAAccount.sol";
import "@openzeppelin/contracts/utils/Create2.sol";

/**
 * @title AAAccountFactory
 * @dev Factory contract for deploying AAAccount instances
 * Uses CREATE2 for deterministic addresses based on owner and salt
 */
contract AAAccountFactory {
    // EntryPoint contract address
    IEntryPoint public immutable entryPoint;
    
    // Events
    event AccountCreated(address indexed account, address indexed owner, uint256 salt);
    event AccountCreatedWithOwners(address indexed account, address[] owners, uint256 salt);
    
    /**
     * @dev Constructor
     * @param _entryPoint EntryPoint contract address
     */
    constructor(IEntryPoint _entryPoint) {
        entryPoint = _entryPoint;
    }
    
    /**
     * @dev Get the address where an account will be deployed
     * @param owner The owner of the account
     * @param salt Unique salt for CREATE2 deployment
     * @return The predicted account address
     */
    function getAddress(address owner, uint256 salt) public view returns (address) {
        return Create2.computeAddress(bytes32(salt), keccak256(abi.encodePacked(
            type(AAAccount).creationCode,
            abi.encode(entryPoint, owner)
        )));
    }
    
    /**
     * @dev Get the address where a multi-owner account will be deployed
     * @param owners Array of owner addresses
     * @param salt Unique salt for CREATE2 deployment
     * @return The predicted account address
     */
    function getAddressWithOwners(address[] calldata owners, uint256 salt) public view returns (address) {
        // Use the same salt modification as in deployment
        bytes32 actualSalt = keccak256(abi.encodePacked(salt, owners));
        return Create2.computeAddress(actualSalt, keccak256(abi.encodePacked(
            type(AAAccount).creationCode,
            abi.encode(entryPoint, address(0)) // Pass zero address, will use initializeWithOwners
        )));
    }
    
    /**
     * @dev Create a new account with a single owner
     * @param owner The owner of the account
     * @param salt Unique salt for CREATE2 deployment
     * @return account The deployed account address
     */
    function createAccount(address owner, uint256 salt) external returns (AAAccount account) {
        require(owner != address(0), "AAAccountFactory: owner cannot be zero");
        
        address addr = getAddress(owner, salt);
        uint256 codeSize = addr.code.length;
        if (codeSize > 0) {
            return AAAccount(payable(addr));
        }
        
        // Deploy using CREATE2 directly
        account = new AAAccount{salt: bytes32(salt)}(entryPoint, owner);
        
        emit AccountCreated(address(account), owner, salt);
    }
    
    /**
     * @dev Create a new account with multiple owners
     * @param owners Array of owner addresses
     * @param salt Unique salt for CREATE2 deployment
     * @return account The deployed account address
     */
    function createAccountWithOwners(address[] calldata owners, uint256 salt) external returns (AAAccount account) {
        require(owners.length > 0, "AAAccountFactory: owners array cannot be empty");
        require(owners.length <= 10, "AAAccountFactory: too many owners (max 10)");
        
        // Validate all owners
        for (uint256 i = 0; i < owners.length; i++) {
            require(owners[i] != address(0), "AAAccountFactory: owner cannot be zero");
            // Check for duplicates
            for (uint256 j = i + 1; j < owners.length; j++) {
                require(owners[i] != owners[j], "AAAccountFactory: duplicate owner");
            }
        }
        
        address addr = getAddressWithOwners(owners, salt);
        uint256 codeSize = addr.code.length;
        if (codeSize > 0) {
            return AAAccount(payable(addr));
        }
        
        // Deploy using CREATE2 directly, then initialize with multiple owners
        bytes32 actualSalt = keccak256(abi.encodePacked(salt, owners));
        account = new AAAccount{salt: actualSalt}(entryPoint, address(0));
        account.initializeWithOwners(owners);
        
        emit AccountCreatedWithOwners(address(account), owners, salt);
    }
    
    /**
     * @dev Check if an account has been deployed
     * @param owner The owner of the account
     * @param salt Unique salt for CREATE2 deployment
     * @return True if the account exists
     */
    function isAccountDeployed(address owner, uint256 salt) external view returns (bool) {
        address predictedAddress = getAddress(owner, salt);
        return predictedAddress.code.length > 0;
    }
    
    /**
     * @dev Check if a multi-owner account has been deployed
     * @param owners Array of owner addresses
     * @param salt Unique salt for CREATE2 deployment
     * @return True if the account exists
     */
    function isAccountWithOwnersDeployed(address[] calldata owners, uint256 salt) external view returns (bool) {
        address predictedAddress = getAddressWithOwners(owners, salt);
        return predictedAddress.code.length > 0;
    }
}
