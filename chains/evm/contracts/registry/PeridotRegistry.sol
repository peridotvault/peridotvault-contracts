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

    mapping(string => GameRecord) public games;

    mapping(address => string) public gameIdOf;

    string[] private _allGameIds;

    address public factory;

    /* ======================================================
       EVENTS
    ====================================================== */

    event FactorySet(address indexed factory);

    event GameRegistered(
        string gameId,
        address indexed pgc1,
        address indexed publisher
    );

    event GameStatusSet(string gameId, bool active);

    /* ======================================================
       ERRORS
    ====================================================== */

    error NotFactory();
    error ZeroAddress();
    error InvalidPGC1();
    error GameAlreadyRegistered();
    error GameNotRegistered();
    error PGC1AlreadyRegistered();
    error InvalidGameId();

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
        string calldata gameId,
        address pgc1,
        address publisher
    ) external onlyFactory {
        if (bytes(gameId).length == 0) revert InvalidGameId();
        if (pgc1 == address(0) || publisher == address(0)) revert ZeroAddress();
        if (pgc1.code.length == 0) revert InvalidPGC1();

        if (games[gameId].pgc1 != address(0)) revert GameAlreadyRegistered();
        if (bytes(gameIdOf[pgc1]).length != 0) revert PGC1AlreadyRegistered();

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

    function setGameActive(
        string calldata gameId,
        bool active
    ) external onlyOwner {
        if (games[gameId].pgc1 == address(0)) revert GameNotRegistered();

        games[gameId].active = active;

        emit GameStatusSet(gameId, active);
    }

    /* ======================================================
       READ HELPERS
    ====================================================== */

    function gameCount() external view returns (uint256) {
        return _allGameIds.length;
    }

    function gameIdAt(uint256 index) external view returns (string memory) {
        return _allGameIds[index];
    }

    function allGameIds() external view returns (string[] memory) {
        return _allGameIds;
    }

    function getGame(
        string calldata gameId
    ) external view returns (GameRecord memory) {
        return games[gameId];
    }
}
