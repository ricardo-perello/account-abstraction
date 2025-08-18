// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";
import "@account-abstraction/contracts/core/BaseAccount.sol";
import "@account-abstraction/contracts/core/Helpers.sol";

contract AAAccount is BaseAccount {
    using ECDSA for bytes32;

    address public owner;
    IEntryPoint private immutable _entryPoint;

    event OwnerChanged(address indexed previousOwner, address indexed newOwner);

    constructor(IEntryPoint anEntryPoint, address initialOwner) {
        _entryPoint = anEntryPoint;
        owner = initialOwner;
    }

    function entryPoint() public view override returns (IEntryPoint) {
        return _entryPoint;
    }

    function setOwner(address newOwner) external {
        require(msg.sender == owner, "AAAccount: only owner");
        require(newOwner != address(0), "AAAccount: zero owner");
        emit OwnerChanged(owner, newOwner);
        owner = newOwner;
    }

    function _requireForExecute() internal view override {
        require(
            msg.sender == address(entryPoint()) || msg.sender == owner,
            "account: not Owner or EntryPoint"
        );
    }

    function _validateSignature(
        PackedUserOperation calldata userOp,
        bytes32 userOpHash
    ) internal override returns (uint256 validationData) {
        // ECDSA recover against EIP-4337 userOpHash (already domain separated by EntryPoint)
        address recovered = ECDSA.recover(userOpHash, userOp.signature);
        if (recovered != owner) {
            return SIG_VALIDATION_FAILED;
        }
        return SIG_VALIDATION_SUCCESS;
    }
}
