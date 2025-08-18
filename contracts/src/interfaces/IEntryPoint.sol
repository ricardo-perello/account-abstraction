// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./UserOperation.sol";

interface IEntryPoint {
    function handleOps(UserOperation[] calldata ops, address payable beneficiary) external;
    function simulateValidation(UserOperation calldata userOp) external;
    function getUserOpHash(UserOperation calldata userOp) external view returns (bytes32);
}
