// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";

contract PeridotRegistry is Ownable {
    struct GameRecord {
        address pgc1;
        address publisher;
        uint64 createdAt;
        bool active;
    }

    /* ======================================================
       STORAGE
    ====================================================== */

    // gameId => record
    mapping(bytes32 => GameRecord) public games;

    // pgc1 => gameId (reverse lookup)
    mapping(address => bytes32) public gameIdOf;

    // iterable index
    bytes32[] private _allGameIds;

    // Only factory can register
    address public factory;

    /* ======================================================
       EVENTS (OPTIONAL, TETAP BAGUS)
    ====================================================== */

    event FactorySet(address indexed factory);
    event GameRegistered(
        bytes32 indexed gameId,
        address indexed pgc1,
        address indexed publisher
    );
    event GameStatusSet(bytes32 indexed gameId, bool active);

    /* ======================================================
       ERRORS
    ====================================================== */

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

    /* ======================================================
       ADMIN
    ====================================================== */

    function setFactory(address factory_) external onlyOwner {
        if (factory_ == address(0)) revert ZeroAddress();
        factory = factory_;
        emit FactorySet(factory_);
    }

    /* ======================================================
       REGISTRATION
    ====================================================== */

    function registerGame(
        bytes32 gameId,
        address pgc1,
        address publisher
    ) external onlyFactory {
        if (pgc1 == address(0) || publisher == address(0)) revert ZeroAddress();
        if (pgc1.code.length == 0) revert InvalidPGC1();

        if (games[gameId].pgc1 != address(0)) revert GameAlreadyRegistered();
        if (gameIdOf[pgc1] != bytes32(0)) revert PGC1AlreadyRegistered();

        games[gameId] = GameRecord({
            pgc1: pgc1,
            publisher: publisher,
            createdAt: uint64(block.timestamp),
            active: true
        });

        gameIdOf[pgc1] = gameId;
        _allGameIds.push(gameId);

        emit GameRegistered(gameId, pgc1, publisher);
    }

    /* ======================================================
       MODERATION
    ====================================================== */

    function setGameActive(bytes32 gameId, bool active) external onlyOwner {
        if (games[gameId].pgc1 == address(0)) revert GameNotRegistered();
        games[gameId].active = active;
        emit GameStatusSet(gameId, active);
    }

    /* ======================================================
       READ HELPERS (INI YANG PENTING UNTUK FRONTEND)
    ====================================================== */

    function gameCount() external view returns (uint256) {
        return _allGameIds.length;
    }

    function gameIdAt(uint256 index) external view returns (bytes32) {
        return _allGameIds[index];
    }

    function allGameIds() external view returns (bytes32[] memory) {
        return _allGameIds;
    }
}
