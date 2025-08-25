// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";
import "@openzeppelin/contracts/utils/cryptography/MessageHashUtils.sol";
import "@openzeppelin/contracts/utils/introspection/IERC165.sol";
import "../lib/account-abstraction/contracts/core/BasePaymaster.sol";

contract VerifierSignaturePaymaster is BasePaymaster {
    using ECDSA for bytes32;
    using MessageHashUtils for bytes32;
    
    // Verifier address that authorizes gas sponsorship
    address public immutable verifier;
    
    // Events for monitoring
    event GasSponsored(address indexed user, uint256 gasCost, bytes32 userOpHash);
    
    constructor(IEntryPoint _entryPoint, address _verifier) BasePaymaster(_entryPoint) {
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
        (bool ok165, bytes memory data165) = ep.staticcall(
            abi.encodeWithSelector(IERC165.supportsInterface.selector, erc165InterfaceId)
        );
        if (ok165 && data165.length == 32 && abi.decode(data165, (bool))) {
            // EntryPoint supports ERC-165, which is good for v0.7+
            return true;
        }

        // 3) Fallback: probe for a well-known EntryPoint function signature
        // Try calling getNonce - a simple view function that should exist
        (bool okNonce,) = ep.staticcall(
            abi.encodeWithSignature("getNonce(address,uint192)", address(0), uint192(0))
        );
        
        return okNonce;
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
        bytes32 /*userOpHash*/,      // ignore - not used for pm sig
        uint256 /*maxCost*/
    ) internal virtual override returns (bytes memory context, uint256 validationData) {
        
        // Layout: [addr (20) | verifGas (16) | postOpGas (16) | ...paymasterData]
        // paymasterData := [signature (65) | validUntil (8) | validAfter (8)]
        bytes calldata d = userOp.paymasterAndData[PAYMASTER_DATA_OFFSET:];
        bytes calldata sig = d[:65];
        uint64 validUntil = uint64(bytes8(d[65:73]));
        uint64 validAfter = uint64(bytes8(d[73:81]));

        bytes32 digest = MessageHashUtils.toEthSignedMessageHash(
            _pmHash(userOp, validUntil, validAfter, 0)
        );
        address recovered = ECDSA.recover(digest, sig);

        bool sigBad = (recovered != verifier);
        if (sigBad) {
            revert("Invalid verifier signature");
        }
        
        // Log gas sponsorship
        emit GasSponsored(userOp.sender, 0, 0); // maxCost not available in this context
        
        // Return success (empty context, 0 validation data)
        return ("", 0);
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
    
    // Allow contract to receive ETH
    receive() external payable {}
}
