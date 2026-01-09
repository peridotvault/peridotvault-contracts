// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {ERC1155} from "@openzeppelin/contracts/token/ERC1155/ERC1155.sol";
import {
    ERC1155Supply
} from "@openzeppelin/contracts/token/ERC1155/extensions/ERC1155Supply.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";
import {
    ReentrancyGuard
} from "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {
    SafeERC20
} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";

import {IPGC1} from "./interfaces/IPGC1.sol";
import {PGC1Constants} from "./lib/PGC1Constants.sol";
import {PGC1Errors} from "./lib/PGC1Errors.sol";
import {PGC1Events} from "./lib/PGC1Events.sol";

contract PGC1 is ERC1155Supply, Ownable, ReentrancyGuard, IPGC1 {
    using SafeERC20 for IERC20;

    // -------------------------
    // Identity
    // -------------------------
    bytes32 public immutable gameId;

    // -------------------------
    // Immutables (config)
    // -------------------------
    address public immutable override paymentToken;
    address public immutable override treasuryRouter;
    address public immutable override developerRecipient;
    uint16 public immutable override platformFeeBps;
    uint256 public immutable override maxSupply; // 0 = unlimited

    // -------------------------
    // Mutable sale config
    // -------------------------
    uint256 public override price;

    // -------------------------
    // Contract-level metadata commits (append-only)
    // -------------------------
    uint32 public override contractMetaHeadVersion; // 0 means none yet
    bytes32 public override contractMetaHeadHash;
    bytes32 public override contractMetaHeadParentHash;

    event ContractMetadataPublished(
        uint32 indexed version,
        bytes32 indexed hash,
        bytes32 indexed parentHash,
        string uri,
        uint64 timestamp
    );

    // -------------------------
    // Game metadata commits (append-only)
    // -------------------------
    uint32 public override metadataHeadVersion; // 0 means none yet
    bytes32 public override metadataHeadHash;
    bytes32 public override metadataHeadParentHash;

    event MetadataPublished(
        uint32 indexed version,
        bytes32 indexed hash,
        bytes32 indexed parentHash,
        string uri,
        uint64 timestamp
    );

    // -------------------------
    // Constructor
    // -------------------------
    constructor(
        string memory tokenURI1155,
        bytes32 initialContractMetaHash,
        string memory initialContractMetaURI,
        bytes32 gameId_,
        address paymentToken_,
        uint256 initialPrice,
        uint256 initialMaxSupply,
        address treasuryRouter_,
        address developerRecipient_,
        uint16 platformFeeBps_
    ) ERC1155(tokenURI1155) Ownable(msg.sender) {
        if (
            treasuryRouter_ == address(0) || developerRecipient_ == address(0)
        ) {
            revert PGC1Errors.ZeroAddress();
        }
        if (platformFeeBps_ > PGC1Constants.PLATFORM_FEE_BPS_CAP) {
            revert PGC1Errors.FeeTooHigh();
        }

        gameId = gameId_;

        paymentToken = paymentToken_;
        treasuryRouter = treasuryRouter_;
        developerRecipient = developerRecipient_;
        platformFeeBps = platformFeeBps_;

        price = initialPrice;
        maxSupply = initialMaxSupply;

        emit PGC1Events.PriceUpdated(initialPrice);
        emit PGC1Events.MaxSupplyInitialized(initialMaxSupply);

        // Publish initial contract-level metadata commit (append-only history starts here)
        _publishContractMetadata(
            initialContractMetaHash,
            initialContractMetaURI
        );
    }

    // -------------------------
    // Identity
    // -------------------------
    function pgcVersion() external pure override returns (uint256) {
        return 1;
    }

    // -------------------------
    // Contract metadata commits (admin)
    // -------------------------
    function publishContractMetadata(
        bytes32 newHash,
        string calldata uri
    ) external override onlyOwner {
        _publishContractMetadata(newHash, uri);
    }

    function _publishContractMetadata(
        bytes32 newHash,
        string memory uri
    ) internal {
        if (newHash == bytes32(0)) revert PGC1Errors.EmptyMetadataHash();
        if (bytes(uri).length == 0) revert PGC1Errors.EmptyMetadataURI();

        bytes32 parent = contractMetaHeadHash;

        unchecked {
            contractMetaHeadVersion += 1;
        }
        contractMetaHeadParentHash = parent;
        contractMetaHeadHash = newHash;

        emit ContractMetadataPublished(
            contractMetaHeadVersion,
            newHash,
            parent,
            uri,
            uint64(block.timestamp)
        );
    }

    // -------------------------
    // Game metadata commits (admin)
    // -------------------------
    function publishMetadata(
        bytes32 newHash,
        string calldata uri
    ) external override onlyOwner {
        if (newHash == bytes32(0)) revert PGC1Errors.EmptyMetadataHash();
        if (bytes(uri).length == 0) revert PGC1Errors.EmptyMetadataURI();

        bytes32 parent = metadataHeadHash;

        unchecked {
            metadataHeadVersion += 1;
        }
        metadataHeadParentHash = parent;
        metadataHeadHash = newHash;

        emit MetadataPublished(
            metadataHeadVersion,
            newHash,
            parent,
            uri,
            uint64(block.timestamp)
        );
    }

    // -------------------------
    // User purchase (self-mint)
    // -------------------------
    function buy() external payable override nonReentrant {
        if (metadataHeadVersion == 0) revert PGC1Errors.NoMetadataPublished();

        if (balanceOf(msg.sender, PGC1Constants.LICENSE_ID) != 0) {
            revert PGC1Errors.AlreadyOwned();
        }

        _enforceCap();
        _mint(msg.sender, PGC1Constants.LICENSE_ID, 1, "");

        if (paymentToken == address(0)) {
            if (msg.value != price) revert PGC1Errors.InvalidPayment();
            _splitEth(price);
        } else {
            if (msg.value != 0) revert PGC1Errors.EthNotAccepted();
            _splitErc20(IERC20(paymentToken), price);
        }

        emit PGC1Events.Purchased(msg.sender, price);
    }

    function burn() external override {
        _burn(msg.sender, PGC1Constants.LICENSE_ID, 1);
    }

    // -------------------------
    // Admin controls
    // -------------------------
    function setPrice(uint256 newPrice) external override onlyOwner {
        price = newPrice;
        emit PGC1Events.PriceUpdated(newPrice);
    }

    // -------------------------
    // Internal helpers
    // -------------------------
    function _enforceCap() internal view {
        uint256 cap = maxSupply;
        if (cap == 0) return;
        if (totalSupply(PGC1Constants.LICENSE_ID) + 1 > cap)
            revert PGC1Errors.SoldOut();
    }

    function _splitEth(uint256 amount) internal {
        uint256 fee = (amount * platformFeeBps) / PGC1Constants.BPS_DENOMINATOR;
        uint256 devAmt = amount - fee;

        (bool ok1, ) = payable(treasuryRouter).call{value: fee}("");
        if (!ok1) revert PGC1Errors.PayoutFailed();

        (bool ok2, ) = payable(developerRecipient).call{value: devAmt}("");
        if (!ok2) revert PGC1Errors.PayoutFailed();
    }

    function _splitErc20(IERC20 token, uint256 amount) internal {
        uint256 fee = (amount * platformFeeBps) / PGC1Constants.BPS_DENOMINATOR;
        uint256 devAmt = amount - fee;

        token.safeTransferFrom(msg.sender, address(this), amount);
        if (fee != 0) token.safeTransfer(treasuryRouter, fee);
        if (devAmt != 0) token.safeTransfer(developerRecipient, devAmt);
    }

    // -------------------------
    // Soulbound enforcement
    // -------------------------
    function _update(
        address from,
        address to,
        uint256[] memory ids,
        uint256[] memory values
    ) internal override(ERC1155Supply) {
        if (from != address(0) && to != address(0)) {
            revert PGC1Errors.NonTransferable();
        }
        super._update(from, to, ids, values);
    }
}
