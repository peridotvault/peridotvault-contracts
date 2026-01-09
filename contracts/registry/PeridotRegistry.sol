// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";

contract PeridotRegistry is Ownable {
    struct GameRecord {
        address pgc1;
        address publisher;
        uint64 createdAt;
        bool active;
    }

    // gameId => record
    mapping(bytes32 => GameRecord) public games;

    // pgc1 => gameId (reverse lookup)
    mapping(address => bytes32) public gameIdOf;

    // Only factory can register (set via owner)
    address public factory;

    event FactorySet(address indexed factory);
    event GameRegistered(
        bytes32 indexed gameId,
        address indexed pgc1,
        address indexed publisher
    );
    event GameStatusSet(bytes32 indexed gameId, bool active);

    error NotFactory();
    error ZeroAddress();
    error InvalidPGC1();
    error GameAlreadyRegistered();
    error GameNotRegistered();
    error PGC1AlreadyRegistered();

    modifier onlyFactory() {
        if (msg.sender != factory) revert NotFactory();
        _;
    }

    constructor() Ownable(msg.sender) {}

    function setFactory(address factory_) external onlyOwner {
        if (factory_ == address(0)) revert ZeroAddress();
        factory = factory_;
        emit FactorySet(factory_);
    }

    function registerGame(
        bytes32 gameId,
        address pgc1,
        address publisher
    ) external onlyFactory {
        if (pgc1 == address(0) || publisher == address(0)) revert ZeroAddress();
        if (pgc1.code.length == 0) revert InvalidPGC1();

        // gameId must be unique
        if (games[gameId].pgc1 != address(0)) revert GameAlreadyRegistered();

        // pgc1 must also be unique
        if (gameIdOf[pgc1] != bytes32(0)) revert PGC1AlreadyRegistered();

        games[gameId] = GameRecord({
            pgc1: pgc1,
            publisher: publisher,
            createdAt: uint64(block.timestamp),
            active: true
        });

        gameIdOf[pgc1] = gameId;

        emit GameRegistered(gameId, pgc1, publisher);
    }

    /// @dev optional moderation / delisting
    function setGameActive(bytes32 gameId, bool active) external onlyOwner {
        if (games[gameId].pgc1 == address(0)) revert GameNotRegistered();
        games[gameId].active = active;
        emit GameStatusSet(gameId, active);
    }
}
