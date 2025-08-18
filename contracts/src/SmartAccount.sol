// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./interfaces/IEntryPoint.sol";
import "./interfaces/UserOperation.sol";

// Skeleton only. No implementation.
abstract contract SmartAccount {
    function validateUserOp(UserOperation calldata userOp, bytes32 userOpHash, uint256 missingAccountFunds)
        external
        virtual
        returns (uint256);

    function execute(address target, uint256 value, bytes calldata data) external virtual;

    function executeBatch(address[] calldata targets, uint256[] calldata values, bytes[] calldata datas)
        external
        virtual;

    function getNonce(uint192 key) external view virtual returns (uint256);

    function isValidSignature(bytes32 hash, bytes calldata signature) public view virtual returns (bool);

    receive() external payable virtual {}
}
