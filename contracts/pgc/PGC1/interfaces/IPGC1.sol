// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

interface IPGC1 {
    // --- immutables ---
    function paymentToken() external view returns (address);
    function treasuryRouter() external view returns (address);
    function developerRecipient() external view returns (address);
    function platformFeeBps() external view returns (uint16);
    function maxSupply() external view returns (uint256);

    // --- sale config ---
    function price() external view returns (uint256);

    // --- identity ---
    function pgcVersion() external pure returns (uint256);

    // --- actions ---
    function buy() external payable;
    function burn() external;

    // --- admin ---
    function setPrice(uint256 newPrice) external;

    // --- contract-level metadata commits (append-only) ---
    function contractMetaHeadVersion() external view returns (uint32);
    function contractMetaHeadHash() external view returns (bytes32);
    function contractMetaHeadParentHash() external view returns (bytes32);
    function publishContractMetadata(
        bytes32 newHash,
        string calldata uri
    ) external;

    // --- game metadata commits (append-only) ---
    function metadataHeadVersion() external view returns (uint32);
    function metadataHeadHash() external view returns (bytes32);
    function metadataHeadParentHash() external view returns (bytes32);
    function publishMetadata(bytes32 newHash, string calldata uri) external;
}
