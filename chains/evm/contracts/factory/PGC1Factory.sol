// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import {Clones} from "@openzeppelin/contracts/proxy/Clones.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {
    SafeERC20
} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";

import {PGC1} from "../pgc/PGC1/PGC1.sol";
import {PGC1Errors} from "../pgc/PGC1/lib/PGC1Errors.sol";
import {IPeridotRegistry} from "../registry/IPeridotRegistry.sol";

/// @title PGC1Factory
/// @notice Permissionless factory for publishing PGC1 game contracts
/// @dev Platform economics & routing enforced by factory
contract PGC1Factory is Ownable {
    using Clones for address;
    using SafeERC20 for IERC20;

    /* ======================================================
       CORE CONFIG
    ====================================================== */

    /// @notice PGC1 logic contract
    address public immutable pgc1Implementation;

    /// @notice global game registry
    IPeridotRegistry public registry;

    /* ======================================================
       PLATFORM ECONOMICS
    ====================================================== */

    /// @notice platform treasury router
    address public treasuryRouter;

    /// @notice platform fee (basis points)
    uint16 public platformFeeBps = 500;

    /// @notice publish fee token (0 = ETH)
    address public feeToken;

    /// @notice publish fee amount
    uint256 public publishFee;

    /* ======================================================
       EVENTS
    ====================================================== */

    event RegistrySet(address indexed registry);

    event TreasuryRouterSet(address indexed router);

    event PlatformFeeBpsSet(uint16 newBps);

    event PublishFeeSet(uint256 newFee);

    event FeeTokenSet(address indexed token);

    event GamePublished(
        address indexed publisher,
        address indexed pgc1,
        string gameId
    );

    /* ======================================================
       ERRORS
    ====================================================== */

    error RegistryNotSet();
    error EthNotAccepted();
    error InvalidPlatformFeeBps();
    error InvalidGameId();

    /* ======================================================
       GAME INIT STRUCT
    ====================================================== */

    struct PGC1Init {
        string tokenURI1155;
        bytes32 initialContractMetaHash;
        string initialContractMetaURI;
        string gameId;
        address paymentToken;
        uint256 price;
        uint256 maxSupply;
    }

    /* ======================================================
       CONSTRUCTOR
    ====================================================== */

    constructor(
        address pgc1Implementation_,
        address treasuryRouter_,
        address feeToken_,
        uint256 publishFee_
    ) Ownable(msg.sender) {
        if (pgc1Implementation_ == address(0)) revert PGC1Errors.ZeroAddress();
        if (treasuryRouter_ == address(0)) revert PGC1Errors.ZeroAddress();

        pgc1Implementation = pgc1Implementation_;
        treasuryRouter = treasuryRouter_;
        feeToken = feeToken_;
        publishFee = publishFee_;

        emit TreasuryRouterSet(treasuryRouter_);
        emit FeeTokenSet(feeToken_);
        emit PublishFeeSet(publishFee_);
    }

    /* ======================================================
       ADMIN
    ====================================================== */

    function setRegistry(address registry_) external onlyOwner {
        if (registry_ == address(0)) revert PGC1Errors.ZeroAddress();

        registry = IPeridotRegistry(registry_);

        emit RegistrySet(registry_);
    }

    function setTreasuryRouter(address newRouter) external onlyOwner {
        if (newRouter == address(0)) revert PGC1Errors.ZeroAddress();

        treasuryRouter = newRouter;

        emit TreasuryRouterSet(newRouter);
    }

    function setPlatformFeeBps(uint16 newBps) external onlyOwner {
        if (newBps > 10_000) revert InvalidPlatformFeeBps();

        platformFeeBps = newBps;

        emit PlatformFeeBpsSet(newBps);
    }

    function setPublishFee(uint256 newFee) external onlyOwner {
        publishFee = newFee;

        emit PublishFeeSet(newFee);
    }

    function setFeeToken(address newToken) external onlyOwner {
        feeToken = newToken;

        emit FeeTokenSet(newToken);
    }

    /* ======================================================
       PUBLISH GAME
    ====================================================== */

    function publishGame(
        PGC1Init calldata init
    ) external payable returns (address pgc1) {
        if (address(registry) == address(0)) revert RegistryNotSet();

        if (bytes(init.gameId).length == 0) revert InvalidGameId();

        _collectPublishFee();

        // deploy minimal proxy
        pgc1 = pgc1Implementation.clone();

        // initialize game contract
        PGC1(pgc1).initialize(
            init.tokenURI1155,
            init.initialContractMetaHash,
            init.initialContractMetaURI,
            init.gameId,
            init.paymentToken,
            init.price,
            init.maxSupply,
            treasuryRouter,
            msg.sender,
            platformFeeBps,
            msg.sender
        );

        // register game in registry
        registry.registerGame(init.gameId, pgc1, msg.sender);

        emit GamePublished(msg.sender, pgc1, init.gameId);
    }

    /* ======================================================
       INTERNAL: PUBLISH FEE
    ====================================================== */

    function _collectPublishFee() internal {
        uint256 fee = publishFee;

        if (fee == 0) return;

        if (feeToken == address(0)) {
            if (msg.value != fee) revert PGC1Errors.InvalidPayment();

            (bool ok, ) = payable(treasuryRouter).call{value: msg.value}("");

            if (!ok) revert PGC1Errors.PayoutFailed();
        } else {
            if (msg.value != 0) revert EthNotAccepted();

            IERC20(feeToken).safeTransferFrom(msg.sender, treasuryRouter, fee);
        }
    }
}
