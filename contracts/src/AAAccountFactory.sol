// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "./AAAccount.sol";
import "@openzeppelin/contracts/utils/Create2.sol";
import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import "@account-abstraction/contracts/core/SenderCreator.sol";

/**
 * @title AAAccountFactory  
 * @dev Factory contract for deploying AAAccount instances using proxy pattern
 * Fully compatible with SimpleAccountFactory interface and aa_sdk_rs
 * Uses CREATE2 for deterministic addresses based on owner and salt
 * Compatible with EntryPoint v0.7+
 */
contract AAAccountFactory {
    /// @notice The EntryPoint contract
    IEntryPoint public immutable entryPoint;

    /// @notice The account implementation contract
    AAAccount public immutable accountImplementation;

    // Events
    event AccountCreated(address indexed account, address indexed owner, uint256 salt);
    event AccountCreatedWithOwners(address indexed account, address[] owners, uint256 salt);

    // Modifiers
    modifier validOwners(address[] calldata owners) {
        require(owners.length > 0 && owners.length <= 10, "AAAccountFactory: invalid owner count");
        
        // Check for duplicates and zero addresses
        for (uint256 i = 0; i < owners.length; i++) {
            require(owners[i] != address(0), "AAAccountFactory: owner cannot be zero");
            for (uint256 j = i + 1; j < owners.length; j++) {
                require(owners[i] != owners[j], "AAAccountFactory: duplicate owner");
            }
        }
        _;
    }

    modifier validSingleOwner(address owner) {
        require(owner != address(0), "AAAccountFactory: owner cannot be zero");
        _;
    }

    constructor(IEntryPoint _entryPoint) {
        require(address(_entryPoint) != address(0), "EntryPoint cannot be zero");
        entryPoint = _entryPoint;
        
        // Deploy the account implementation
        accountImplementation = new AAAccount();
        
        // In v0.7.0, EntryPoint doesn't have senderCreator function
        // We'll use direct account creation instead
    }
    
    /**
     * @dev Get the address where an account will be deployed
     * @param owner The owner of the account
     * @param salt Unique salt for CREATE2 deployment
     * @return The predicted account address
     */
    function getAddress(address owner, uint256 salt) public view returns (address) {
        return Create2.computeAddress(bytes32(salt), keccak256(abi.encodePacked(
            type(ERC1967Proxy).creationCode,
            abi.encode(
                address(accountImplementation),
                abi.encodeCall(AAAccount.initialize, (owner, address(entryPoint)))
            )
        )));
    }
    
    /**
     * @dev Get the address where a multi-owner account will be deployed
     * @param owners Array of owner addresses
     * @param salt Unique salt for CREATE2 deployment
     * @return The predicted account address
     */
    function getAddressWithOwners(address[] calldata owners, uint256 salt) public view returns (address) {
        // Create deterministic salt based on owners array for uniqueness
        bytes32 actualSalt = _computeMultiOwnerSalt(salt, owners);
        return Create2.computeAddress(actualSalt, keccak256(abi.encodePacked(
            type(ERC1967Proxy).creationCode,
            abi.encode(
                address(accountImplementation),
                abi.encodeCall(AAAccount.initializeWithOwners, (owners, address(entryPoint)))
            )
        )));
    }

    /**
     * @dev Compute deterministic salt for multi-owner accounts
     * @param baseSalt Base salt provided by user
     * @param owners Array of owner addresses
     * @return Computed salt that ensures uniqueness
     */
    function _computeMultiOwnerSalt(uint256 baseSalt, address[] calldata owners) internal pure returns (bytes32) {
        return keccak256(abi.encodePacked(baseSalt, owners));
    }
    
    /**
     * @dev Create a new account with a single owner
     * Compatible with SimpleAccountFactory interface
     * @param owner The owner of the account
     * @param salt Unique salt for CREATE2 deployment
     * @return ret The deployed account address
     */
    function createAccount(address owner, uint256 salt) public returns (AAAccount ret) {
        // Only enforce SenderCreator restriction if it's available
        // In v0.7.0, EntryPoint doesn't have senderCreator function
        // This check is no longer relevant as senderCreator is removed.
        // The account creation logic will now always work directly.
        
        address addr = getAddress(owner, salt);
        uint256 codeSize = addr.code.length;
        if (codeSize > 0) {
            return AAAccount(payable(addr));
        }
        ret = AAAccount(payable(new ERC1967Proxy{salt : bytes32(salt)}(
                address(accountImplementation),
                abi.encodeCall(AAAccount.initialize, (owner, address(entryPoint)))
            )));
        
        emit AccountCreated(address(ret), owner, salt);
    }
    
    /**
     * @dev Create a new account with multiple owners
     * @param owners Array of owner addresses
     * @param salt Unique salt for CREATE2 deployment
     * @return account The deployed account address
     */
    function createAccountWithOwners(address[] calldata owners, uint256 salt) external validOwners(owners) returns (AAAccount account) {
        address addr = getAddressWithOwners(owners, salt);
        uint256 codeSize = addr.code.length;
        if (codeSize > 0) {
            return AAAccount(payable(addr));
        }
        
        // Deploy using CREATE2 with proxy pattern
        bytes32 actualSalt = _computeMultiOwnerSalt(salt, owners);
        account = AAAccount(payable(new ERC1967Proxy{salt : actualSalt}(
                address(accountImplementation),
                abi.encodeCall(AAAccount.initializeWithOwners, (owners, address(entryPoint)))
            )));
        
        emit AccountCreatedWithOwners(address(account), owners, salt);
    }
    
    /**
     * @dev Create account directly (without SenderCreator restriction)
     * For direct deployment when not using UserOperations
     * @param owner The owner of the account
     * @param salt Unique salt for CREATE2 deployment
     * @return account The deployed account address
     */
    function createAccountDirect(address owner, uint256 salt) external validSingleOwner(owner) returns (AAAccount account) {
        address addr = getAddress(owner, salt);
        uint256 codeSize = addr.code.length;
        if (codeSize > 0) {
            return AAAccount(payable(addr));
        }
        account = AAAccount(payable(new ERC1967Proxy{salt : bytes32(salt)}(
                address(accountImplementation),
                abi.encodeCall(AAAccount.initialize, (owner, address(entryPoint)))
            )));
        
        emit AccountCreated(address(account), owner, salt);
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
