// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

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

    /* ======================================================
       CORE CONFIG
    ====================================================== */

    string public gameId;

    address public override paymentToken;
    address public override treasuryRouter;
    address public override developerRecipient;
    uint16 public override platformFeeBps;
    uint256 public override maxSupply; // 0 = unlimited
    uint256 public override price;

    /// H3: explicit on-chain free marker. Default `false` (paid, fail-closed).
    /// `buy` with zero payment requires `isFree == true`; payment requires `false`.
    bool public override isFree;

    string private _tokenBaseURI;

    bool private _initialized;

    /* ======================================================
       METADATA COMMIT STRUCT
    ====================================================== */

    struct MetadataCommit {
        bytes32 hash;
        bytes32 parentHash;
        uint64 timestamp;
        string uri;
    }

    /* ======================================================
       CONTRACT-LEVEL METADATA (ITERABLE)
    ====================================================== */

    MetadataCommit[] private _contractMetadataHistory;

    uint32 public override contractMetaHeadVersion;
    bytes32 public override contractMetaHeadHash;
    bytes32 public override contractMetaHeadParentHash;

    event ContractMetadataPublished(
        uint32 indexed version,
        bytes32 indexed hash,
        bytes32 indexed parentHash,
        string uri,
        uint64 timestamp
    );

    /* ======================================================
       GAME METADATA (ITERABLE)
    ====================================================== */

    MetadataCommit[] private _metadataHistory;

    uint32 public override metadataHeadVersion;
    bytes32 public override metadataHeadHash;
    bytes32 public override metadataHeadParentHash;

    event MetadataPublished(
        uint32 indexed version,
        bytes32 indexed hash,
        bytes32 indexed parentHash,
        string uri,
        uint64 timestamp
    );

    /* ======================================================
       PURCHASE TRACKING (OPTIONAL UI INDEX)
    ====================================================== */

    address[] private _buyers;
    mapping(address => uint64) public purchasedAt;

    /* ======================================================
       CONSTRUCTOR
    ====================================================== */

    constructor() ERC1155("") Ownable(msg.sender) {}

    /* ======================================================
       ERC1155 METADATA
    ====================================================== */

    function uri(uint256) public view override returns (string memory) {
        return _tokenBaseURI;
    }

    /* ======================================================
       INITIALIZER (CLONE-READY)
    ====================================================== */

    function initialize(
        string calldata tokenURI1155,
        bytes32 initialContractMetaHash,
        string calldata initialContractMetaURI,
        string calldata gameId_,
        address paymentToken_,
        uint256 initialPrice,
        uint256 initialMaxSupply,
        address treasuryRouter_,
        address developerRecipient_,
        uint16 platformFeeBps_,
        address owner_
    ) external {
        if (_initialized) revert PGC1Errors.AlreadyInitialized();
        _initialized = true;

        if (treasuryRouter_ == address(0) || developerRecipient_ == address(0))
            revert PGC1Errors.ZeroAddress();

        if (platformFeeBps_ > PGC1Constants.PLATFORM_FEE_BPS_CAP)
            revert PGC1Errors.FeeTooHigh();

        gameId = gameId_;
        paymentToken = paymentToken_;
        treasuryRouter = treasuryRouter_;
        developerRecipient = developerRecipient_;
        platformFeeBps = platformFeeBps_;

        price = initialPrice;
        // H3: paid by default. A zero initial price does NOT imply free;
        // the owner must explicitly call `setIsFree(true)`.
        isFree = false;
        maxSupply = initialMaxSupply;
        _tokenBaseURI = tokenURI1155;

        _transferOwnership(owner_);

        emit PGC1Events.PriceUpdated(initialPrice);
        emit PGC1Events.MaxSupplyInitialized(initialMaxSupply);

        _publishContractMetadata(
            initialContractMetaHash,
            initialContractMetaURI
        );
    }

    /* ======================================================
       VERSION
    ====================================================== */

    function pgcVersion() external pure override returns (uint256) {
        return 1;
    }

    /// M5: on-chain contract schema version (mirrors `pgcVersion`).
    function contractVersion() external pure returns (uint8) {
        return PGC1Constants.CONTRACT_VERSION;
    }

    /// M5: purchase-attestation envelope version (forward-compat with the
    /// EIP-712 `PGL1-XCHAIN` envelope planned for M2-prep).
    function attestationVersion() external pure returns (uint8) {
        return PGC1Constants.ATTESTATION_VERSION;
    }

    /* ======================================================
       CONTRACT METADATA (ADMIN)
    ====================================================== */

    function publishContractMetadata(
        bytes32 newHash,
        string calldata uri_
    ) external override onlyOwner {
        _publishContractMetadata(newHash, uri_);
    }

    function _publishContractMetadata(
        bytes32 newHash,
        string memory uri_
    ) internal {
        if (newHash == bytes32(0)) revert PGC1Errors.EmptyMetadataHash();
        if (bytes(uri_).length == 0) revert PGC1Errors.EmptyMetadataURI();

        bytes32 parent = contractMetaHeadHash;

        unchecked {
            contractMetaHeadVersion += 1;
        }

        contractMetaHeadParentHash = parent;
        contractMetaHeadHash = newHash;

        _contractMetadataHistory.push(
            MetadataCommit({
                hash: newHash,
                parentHash: parent,
                timestamp: uint64(block.timestamp),
                uri: uri_
            })
        );

        emit ContractMetadataPublished(
            contractMetaHeadVersion,
            newHash,
            parent,
            uri_,
            uint64(block.timestamp)
        );
    }

    /* ======================================================
       GAME METADATA (ADMIN)
    ====================================================== */

    function publishMetadata(
        bytes32 newHash,
        string calldata uri_
    ) external override onlyOwner {
        if (newHash == bytes32(0)) revert PGC1Errors.EmptyMetadataHash();
        if (bytes(uri_).length == 0) revert PGC1Errors.EmptyMetadataURI();

        bytes32 parent = metadataHeadHash;

        unchecked {
            metadataHeadVersion += 1;
        }

        metadataHeadParentHash = parent;
        metadataHeadHash = newHash;

        _metadataHistory.push(
            MetadataCommit({
                hash: newHash,
                parentHash: parent,
                timestamp: uint64(block.timestamp),
                uri: uri_
            })
        );

        emit MetadataPublished(
            metadataHeadVersion,
            newHash,
            parent,
            uri_,
            uint64(block.timestamp)
        );
    }

    /* ======================================================
       PURCHASE
    ====================================================== */

    function buy() external payable override nonReentrant {
        if (balanceOf(msg.sender, PGC1Constants.LICENSE_ID) != 0)
            revert PGC1Errors.AlreadyOwned();

        _enforceCap();

        // H2 (Access control): `buy` is the SOLE external mint entry point —
        // there is no public `mint()` on clones or the implementation (the
        // only `_mint` call in this contract is below). Unauthorized direct
        // minting is impossible by construction; payment enforcement is the
        // minter gate (permissionless purchase model: payment enforcement is
        // the minter gate; no MINTER_ROLE by design).
        // H3 two-sided free/paid invariant (fail-closed, flag authoritative):
        if (isFree) {
            // Free path: zero settlement only. Any payment attempt fails
            // (no ambiguous middle).
            if (paymentToken == address(0)) {
                if (msg.value != 0) revert PGC1Errors.FreeGamePaymentNotAllowed();
            } else {
                if (msg.value != 0) revert PGC1Errors.EthNotAccepted();
            }
            if (price != 0) revert PGC1Errors.FreeGamePaymentNotAllowed();
        } else {
            // Paid path: exact payment required (zero payment reverts).
            if (paymentToken == address(0)) {
                if (msg.value != price) revert PGC1Errors.InvalidPayment();
                if (price == 0) revert PGC1Errors.FreePathNotAllowed();
            } else {
                if (msg.value != 0) revert PGC1Errors.EthNotAccepted();
                if (price == 0) revert PGC1Errors.FreePathNotAllowed();
            }
        }

        _mint(msg.sender, PGC1Constants.LICENSE_ID, 1, "");

        if (purchasedAt[msg.sender] == 0) {
            purchasedAt[msg.sender] = uint64(block.timestamp);
            _buyers.push(msg.sender);
        }

        // Free path settles zero (skip token/ETH movement entirely);
        // paid path splits exactly `price` (re-checked below).
        if (!isFree) {
            if (paymentToken == address(0)) {
                if (msg.value != price) revert PGC1Errors.InvalidPayment();
                _splitEth(price);
            } else {
                if (msg.value != 0) revert PGC1Errors.EthNotAccepted();
                _splitErc20(IERC20(paymentToken), price);
            }
        }

        emit PGC1Events.Purchased(msg.sender, price);
    }

    function burn() external override {
        _burn(msg.sender, PGC1Constants.LICENSE_ID, 1);
    }

    /* ======================================================
       ADMIN
    ====================================================== */

    function setPrice(uint256 newPrice) external override onlyOwner {
        // H3: price and free flag must stay consistent (no ambiguous middle).
        // Free games are always price 0; paid games always price > 0.
        if (isFree) {
            if (newPrice != 0) revert PGC1Errors.FreeGamePaymentNotAllowed();
        } else {
            if (newPrice == 0) revert PGC1Errors.FreePathNotAllowed();
        }
        price = newPrice;
        emit PGC1Events.PriceUpdated(newPrice);
    }

    /// H3: explicitly mark a game free (price forced to 0) or paid.
    /// Only the owner may flip the flag. Marking free while `price != 0`
    /// resets price to 0 atomically; marking paid requires the caller to
    /// set a non-zero price afterwards via `setPrice`.
    /// No mirror guard on `paymentToken`: the token address is immutable
    /// after `initialize` (no setter), so rejecting `free == true` while it
    /// is set would make ERC20 games permanently un-freeable. The stale
    /// config is inert — the free path in `buy` settles zero and moves no
    /// tokens — so flipping free harms only the publisher's own revenue
    /// (self-harm only, owner-gated, no third-party funds at risk).
    /// Recovery is `setIsFree(false)` + `setPrice(nz)`.
    function setIsFree(bool free_) external onlyOwner {
        isFree = free_;
        if (free_) {
            if (price != 0) {
                price = 0;
                emit PGC1Events.PriceUpdated(0);
            }
        }
        emit PGC1Events.FreeFlagUpdated(free_);
    }

    /* ======================================================
       READ HELPERS (UI-FRIENDLY)
    ====================================================== */

    function contractMetadataCount() external view returns (uint256) {
        return _contractMetadataHistory.length;
    }

    function contractMetadataAt(
        uint256 index
    ) external view returns (MetadataCommit memory) {
        return _contractMetadataHistory[index];
    }

    function metadataCount() external view returns (uint256) {
        return _metadataHistory.length;
    }

    function metadataAt(
        uint256 index
    ) external view returns (MetadataCommit memory) {
        return _metadataHistory[index];
    }

    function buyerCount() external view returns (uint256) {
        return _buyers.length;
    }

    function buyerAt(uint256 index) external view returns (address) {
        return _buyers[index];
    }

    /* ======================================================
       INTERNAL HELPERS
    ====================================================== */

    function _enforceCap() internal view {
        uint256 cap = maxSupply;
        if (cap == 0) return;
        if (totalSupply(PGC1Constants.LICENSE_ID) + 1 > cap)
            revert PGC1Errors.SoldOut();
    }

    function _splitEth(uint256 amount) internal {
        // M2 native-withdraw hardening: skip zero settlement (no-op).
        if (amount == 0) return;
        uint256 fee = (amount * platformFeeBps) / PGC1Constants.BPS_DENOMINATOR;

        uint256 devAmt = amount - fee;

        // Checks-effects-interactions: state (`_mint`, balances) already
        // updated before this call; `nonReentrant` on `buy` blocks reentry.
        // Zero addresses impossible: set at `initialize` (ZeroAddress gate).
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

    /* ======================================================
       SOULBOUND ENFORCEMENT
    ====================================================== */

    function _update(
        address from,
        address to,
        uint256[] memory ids,
        uint256[] memory values
    ) internal override(ERC1155Supply) {
        if (from != address(0) && to != address(0))
            revert PGC1Errors.NonTransferable();
        super._update(from, to, ids, values);
    }
}
