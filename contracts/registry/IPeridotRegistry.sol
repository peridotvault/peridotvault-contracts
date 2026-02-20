// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

interface IPeridotRegistry {
    function registerGame(
        bytes32 gameId,
        address pgc1,
        address publisher
    ) external;

    function games(
        bytes32 gameId
    )
        external
        view
        returns (
            address pgc1,
            address publisher,
            uint64 createdAt,
            bool active
        );

    function gameIdOf(address pgc1) external view returns (bytes32);

    function factory() external view returns (address);
}
