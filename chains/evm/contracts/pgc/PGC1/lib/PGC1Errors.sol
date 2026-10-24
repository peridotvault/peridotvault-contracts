// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

library PGC1Errors {
    error AlreadyOwned();
    error AlreadyInitialized();

    error SoldOut();
    error InvalidPayment();
    error EthNotAccepted();

    error ZeroAddress();
    error FeeTooHigh();
    error CapBelowMinted();
    error PayoutFailed();

    error NonTransferable();

    error EmptyMetadataHash();
    error EmptyMetadataURI();
    error NoMetadataPublished();

    // M0 H3: free/paid path violations.
    error FreePathNotAllowed();
    error FreeGamePaymentNotAllowed();

    // M0 M5: attestation/envelope version gate.
    error UnsupportedVersion();
}
