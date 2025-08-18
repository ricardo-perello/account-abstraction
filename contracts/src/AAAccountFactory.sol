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
    
    // Template account for CREATE2 deployment
    AAAccount public immutable accountImplementation;
    
    // Events
    event AccountCreated(address indexed account, address indexed owner, uint256 salt);
    event AccountCreatedWithOwners(address indexed account, address[] owners, uint256 salt);
    
    /**
     * @dev Constructor
     * @param _entryPoint EntryPoint contract address
     */
    constructor(IEntryPoint _entryPoint) {
        entryPoint = _entryPoint;
        
        // Deploy the implementation contract
        accountImplementation = new AAAccount(_entryPoint, address(0));
    }
    
    /**
     * @dev Create a new account with a single owner
     * @param owner The owner of the account
     * @param salt Unique salt for CREATE2 deployment
     * @return account The deployed account address
     */
    function createAccount(address owner, bytes32 salt) external returns (AAAccount account) {
        require(owner != address(0), "AAAccountFactory: owner cannot be zero");
        
        // Encode the constructor parameters
        bytes memory initCode = abi.encodePacked(
            address(accountImplementation),
            abi.encodeWithSelector(AAAccount.initialize.selector, owner)
        );
        
        // Deploy using CREATE2
        address deployedAddress = Create2.deploy(
            0, // amount of ETH to send
            salt,
            initCode
        );
        
        account = AAAccount(payable(deployedAddress));
        
        emit AccountCreated(address(account), owner, uint256(salt));
    }
    
    /**
     * @dev Create a new account with multiple owners
     * @param owners Array of owner addresses
     * @param salt Unique salt for CREATE2 deployment
     * @return account The deployed account address
     */
    function createAccountWithOwners(address[] calldata owners, bytes32 salt) external returns (AAAccount account) {
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
        
        // Encode the constructor parameters for multi-owner initialization
        bytes memory initCode = abi.encodePacked(
            address(accountImplementation),
            abi.encodeWithSelector(AAAccount.initializeWithOwners.selector, owners)
        );
        
        // Deploy using CREATE2
        address deployedAddress = Create2.deploy(
            0, // amount of ETH to send
            salt,
            initCode
        );
        
        account = AAAccount(payable(deployedAddress));
        
        emit AccountCreatedWithOwners(address(account), owners, uint256(salt));
    }
    
    /**
     * @dev Get the address where an account will be deployed
     * @param owner The owner of the account
     * @param salt Unique salt for CREATE2 deployment
     * @return The predicted account address
     */
    function getAddress(address owner, bytes32 salt) external view returns (address) {
        bytes memory initCode = abi.encodePacked(
            address(accountImplementation),
            abi.encodeWithSelector(AAAccount.initialize.selector, owner)
        );
        
        bytes32 hash = keccak256(abi.encodePacked(
            bytes1(0xff),
            address(this),
            salt,
            keccak256(initCode)
        ));
        
        return address(uint160(uint256(hash)));
    }
    
    /**
     * @dev Get the address where a multi-owner account will be deployed
     * @param owners Array of owner addresses
     * @param salt Unique salt for CREATE2 deployment
     * @return The predicted account address
     */
    function getAddressWithOwners(address[] calldata owners, bytes32 salt) external view returns (address) {
        bytes memory initCode = abi.encodePacked(
            address(accountImplementation),
            abi.encodeWithSelector(AAAccount.initializeWithOwners.selector, owners)
        );
        
        bytes32 hash = keccak256(abi.encodePacked(
            bytes1(0xff),
            address(this),
            salt,
            keccak256(initCode)
        ));
        
        return address(uint160(uint256(hash)));
    }
    
    /**
     * @dev Check if an account has been deployed
     * @param owner The owner of the account
     * @param salt Unique salt for CREATE2 deployment
     * @return True if the account exists
     */
    function isAccountDeployed(address owner, bytes32 salt) external view returns (bool) {
        address predictedAddress = this.getAddress(owner, salt);
        return predictedAddress.code.length > 0;
    }
    
    /**
     * @dev Check if a multi-owner account has been deployed
     * @param owners Array of owner addresses
     * @param salt Unique salt for CREATE2 deployment
     * @return True if the account exists
     */
    function isAccountWithOwnersDeployed(address[] calldata owners, bytes32 salt) external view returns (bool) {
        address predictedAddress = this.getAddressWithOwners(owners, salt);
        return predictedAddress.code.length > 0;
    }
}
