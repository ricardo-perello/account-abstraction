// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";
import "@openzeppelin/contracts/utils/cryptography/MessageHashUtils.sol";
import "@openzeppelin/contracts/utils/introspection/IERC165.sol";
import "../lib/account-abstraction/contracts/core/BasePaymaster.sol";
import "../lib/account-abstraction/contracts/core/Helpers.sol";

contract VerifierSignaturePaymaster is BasePaymaster {
    using ECDSA for bytes32;
    using MessageHashUtils for bytes32;
    
    // Verifier address that authorizes gas sponsorship
    address public immutable verifier;
    
    // Events for monitoring
    event GasSponsored(address indexed user, uint256 actualGasCost, bytes32 userOpHash);
    
    constructor(IEntryPoint _entryPoint, address _verifier) BasePaymaster(_entryPoint) {
        require(_verifier != address(0), "verifier=0");
        verifier = _verifier;
        // Skip interface validation for deployment on networks where EntryPoint doesn't implement ERC165
    }
    
    // Override the interface validation with robust checking
    function _validateEntryPointInterface(IEntryPoint _entryPoint) internal view override {
        // Robust EntryPoint validation that handles v0.7 combined interfaces
        require(_entryPointLooksValid(address(_entryPoint)), "EntryPoint validation failed");
    }
    
    // Robust EntryPoint validation (handles ERC-165 + fallback to selector probes)
    function _entryPointLooksValid(address ep) internal view returns (bool) {
        // 1) Code must exist
        if (ep.code.length == 0) return false;

        // 2) Try ERC-165: supports ERC165 interface itself?
        bytes4 erc165InterfaceId = 0x01ffc9a7;
        (bool ok165, bytes memory data165) =
            ep.staticcall(abi.encodeWithSelector(IERC165.supportsInterface.selector, erc165InterfaceId));
        if (ok165 && data165.length == 32 && abi.decode(data165, (bool))) {
            // If ERC-165 is present, also require the IEntryPoint interfaceId (v0.7+ EPs expose it)
            (bool okEP, bytes memory dataEP) =
                ep.staticcall(abi.encodeWithSelector(IERC165.supportsInterface.selector, type(IEntryPoint).interfaceId));
            if (okEP && dataEP.length == 32 && abi.decode(dataEP, (bool))) return true;
            // Fall through to selector probe to tolerate minor version drift
        }

        // 3) Fallback: probe for a well-known EntryPoint function signature
        // Try calling getNonce - a simple view function that should exist.
        // Be strict about returndata to avoid false positives from generic fallback handlers.
        (bool okNonce, bytes memory ret) =
            ep.staticcall(abi.encodeWithSignature("getNonce(address,uint192)", address(0), uint192(0)));
        return okNonce && ret.length >= 32;
    }
    
    // v0.7-style PackedUserOperation packing for paymaster digest
    function _packForPaymaster(PackedUserOperation calldata u)
        internal pure returns (bytes memory)
    {
        return abi.encode(
            u.sender,
            u.nonce,
            keccak256(u.initCode),
            keccak256(u.callData),
            u.accountGasLimits,        // bytes32 (packed call/verification gas)
            u.preVerificationGas,      // uint256
            u.gasFees                  // bytes32 (packed maxPriority/maxFee)
        );
    }

    function _pmHash(
        PackedUserOperation calldata u,
        uint64 validUntil,
        uint64 validAfter,           // use 0 if unused
        uint256 /*maxCost*/          // don't bind to EntryPoint's computed value
    ) internal view returns (bytes32) {
        return keccak256(
            abi.encode(
                _packForPaymaster(u),
                block.chainid,
                address(this),        // bind to *this* paymaster
                validUntil,
                validAfter
            )
        );
    }

    function _validatePaymasterUserOp(
        PackedUserOperation calldata userOp,
        bytes32 userOpHash,
        uint256 /*maxCost*/
    ) internal virtual override returns (bytes memory context, uint256 validationData) {
        // ERC-4337 v0.7: paymasterAndData contains ONLY our data (no concatenation)
        // EntryPoint sends paymaster address and gas limits as separate fields
        bytes calldata d = userOp.paymasterAndData;
        // Accept 64- or 65-byte signatures + 16 bytes for time bounds
        require(d.length >= 64 + 8 + 8, "paymasterData short");
        uint256 sigLenCandidate = d.length >= 65 + 16 ? 65 : 64;
        require(d.length >= sigLenCandidate + 16, "paymasterData short");
        bytes calldata sig = d[:sigLenCandidate];
        uint64 validUntil = uint64(bytes8(d[sigLenCandidate : sigLenCandidate + 8]));
        uint64 validAfter = uint64(bytes8(d[sigLenCandidate + 8 : sigLenCandidate + 16]));

        bytes32 digest = MessageHashUtils.toEthSignedMessageHash(
            _pmHash(userOp, validUntil, validAfter, 0)
        );
        if (ECDSA.recover(digest, sig) != verifier) {
            // Signal signature failure via validationData bit instead of reverting
            return ("", SIG_VALIDATION_FAILED);
        }

        // Provide data we want in postOp (e.g., userOpHash, sender)
        context = abi.encode(userOpHash, userOp.sender);
        // Pack validation data: sigFailed(1) + validUntil(6) + validAfter(6) bytes  
        validationData = uint256(validAfter) | (uint256(validUntil) << 48);
        return (context, validationData);
    }
    
    // Public helper for off-chain signature creation
    function getPaymasterHash(
        PackedUserOperation calldata userOp,
        uint64 validUntil,
        uint64 validAfter
    ) external view returns (bytes32) {
        return _pmHash(userOp, validUntil, validAfter, 0);
    }
    
    // Public helper for off-chain signature creation with EIP-191 formatting
    function getPaymasterDigest(
        PackedUserOperation calldata userOp,
        uint64 validUntil,
        uint64 validAfter
    ) external view returns (bytes32) {
        return MessageHashUtils.toEthSignedMessageHash(
            _pmHash(userOp, validUntil, validAfter, 0)
        );
    }
    
    // Allow contract to receive ETH -> immediately deposit to EntryPoint
    receive() external payable {
        entryPoint.depositTo{value: msg.value}(address(this));
    }

    // Implement BasePaymaster's virtual _postOp method
    function _postOp(
        PostOpMode mode,
        bytes calldata context,
        uint256 actualGasCost,
        uint256 actualUserOpFeePerGas
    ) internal override {
        (bytes32 userOpHash, address user) = abi.decode(context, (bytes32, address));
        emit GasSponsored(user, actualGasCost, userOpHash);
        // optional: additional accounting/refund logic
    }
}
