// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

library PGC1Errors {
    error AlreadyOwned();
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
}
