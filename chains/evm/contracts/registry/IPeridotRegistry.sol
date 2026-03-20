// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

interface IPeridotRegistry {
    function registerGame(
        string calldata gameId,
        address pgc1,
        address publisher
    ) external;

    function games(
        string calldata gameId
    )
        external
        view
        returns (
            address pgc1,
            address publisher,
            uint64 createdAt,
            bool active
        );

    function gameIdOf(address pgc1) external view returns (string memory);

    function factory() external view returns (address);
}
