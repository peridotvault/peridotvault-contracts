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
/// @dev Platform economics & routing are enforced at factory-level
contract PGC1Factory is Ownable {
    using Clones for address;
    using SafeERC20 for IERC20;

    // =============================================================
    // Core Config
    // =============================================================

    /// @notice PGC1 implementation address (logic contract)
    address public immutable pgc1Implementation;

    /// @notice Global on-chain registry
    IPeridotRegistry public registry;

    // =============================================================
    // Platform Economics (ENFORCED)
    // =============================================================

    /// @notice Platform treasury router (can be same as feeRecipient)
    address public treasuryRouter;

    /// @notice Platform fee in basis points (0–1_000)
    uint16 public platformFeeBps = 1_000;

    /// @notice Publish fee token (address(0) = ETH)
    address public feeToken;

    /// @notice Publish fee amount
    uint256 public publishFee;

    // =============================================================
    // Events
    // =============================================================

    event RegistrySet(address indexed registry);
    event TreasuryRouterSet(address indexed router);
    event PlatformFeeBpsSet(uint16 newBps);

    event PublishFeeSet(uint256 newFee);
    event FeeRecipientSet(address indexed newRecipient);
    event FeeTokenSet(address indexed newToken);

    event GamePublished(
        address indexed publisher,
        address indexed pgc1,
        bytes32 indexed gameId
    );

    // =============================================================
    // Errors
    // =============================================================

    error RegistryNotSet();
    error EthNotAccepted();
    error InvalidPlatformFeeBps();

    // =============================================================
    // Init Struct (USER-SAFE)
    // =============================================================

    /// @notice Parameters provided by developer (NO platform control here)
    struct PGC1Init {
        string tokenURI1155;
        bytes32 initialContractMetaHash;
        string initialContractMetaURI;
        bytes32 gameId;
        address paymentToken;
        uint256 price;
        uint256 maxSupply;
    }

    // =============================================================
    // Constructor
    // =============================================================

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

    // =============================================================
    // Admin: Registry
    // =============================================================

    function setRegistry(address registry_) external onlyOwner {
        if (registry_ == address(0)) revert PGC1Errors.ZeroAddress();
        registry = IPeridotRegistry(registry_);
        emit RegistrySet(registry_);
    }

    // =============================================================
    // Admin: Platform Economics
    // =============================================================

    function setPlatformFeeBps(uint16 newBps) external onlyOwner {
        if (newBps > 10_000) revert InvalidPlatformFeeBps();
        platformFeeBps = newBps;
        emit PlatformFeeBpsSet(newBps);
    }

    function setTreasuryRouter(address newRouter) external onlyOwner {
        if (newRouter == address(0)) revert PGC1Errors.ZeroAddress();
        treasuryRouter = newRouter;
        emit TreasuryRouterSet(newRouter);
    }

    function setPublishFee(uint256 newFee) external onlyOwner {
        publishFee = newFee;
        emit PublishFeeSet(newFee);
    }

    function setFeeToken(address newToken) external onlyOwner {
        feeToken = newToken;
        emit FeeTokenSet(newToken);
    }

    // =============================================================
    // Publish (PERMISSIONLESS)
    // =============================================================

    function publishGame(
        PGC1Init calldata init
    ) external payable returns (address pgc1) {
        if (address(registry) == address(0)) revert RegistryNotSet();
        if (treasuryRouter == address(0)) revert PGC1Errors.ZeroAddress();

        _collectPublishFee();

        // Deploy minimal proxy
        pgc1 = pgc1Implementation.clone();

        // Initialize game contract
        PGC1(pgc1).initialize(
            init.tokenURI1155,
            init.initialContractMetaHash,
            init.initialContractMetaURI,
            init.gameId,
            init.paymentToken,
            init.price,
            init.maxSupply,
            treasuryRouter, // 🔒 platform-controlled
            msg.sender, // 🔒 developerRecipient
            platformFeeBps, // 🔒 platform-controlled
            msg.sender // owner
        );

        // Register atomically
        registry.registerGame(init.gameId, pgc1, msg.sender);

        emit GamePublished(msg.sender, pgc1, init.gameId);
    }

    // =============================================================
    // Internal: Publish Fee
    // =============================================================

    function _collectPublishFee() internal {
        uint256 fee = publishFee;
        if (fee == 0) return;

        if (feeToken == address(0)) {
            // ETH mode
            if (msg.value != fee) revert PGC1Errors.InvalidPayment();

            (bool ok, ) = payable(treasuryRouter).call{value: msg.value}("");
            if (!ok) revert PGC1Errors.PayoutFailed();
        } else {
            // ERC20 mode
            if (msg.value != 0) revert EthNotAccepted();
            IERC20(feeToken).safeTransferFrom(msg.sender, treasuryRouter, fee);
        }
    }
}
