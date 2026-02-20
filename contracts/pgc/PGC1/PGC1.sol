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

    bytes32 public gameId;

    address public override paymentToken;
    address public override treasuryRouter;
    address public override developerRecipient;
    uint16 public override platformFeeBps;
    uint256 public override maxSupply; // 0 = unlimited
    uint256 public override price;

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
        bytes32 gameId_,
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
        _mint(msg.sender, PGC1Constants.LICENSE_ID, 1, "");

        if (purchasedAt[msg.sender] == 0) {
            purchasedAt[msg.sender] = uint64(block.timestamp);
            _buyers.push(msg.sender);
        }

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

    /* ======================================================
       ADMIN
    ====================================================== */

    function setPrice(uint256 newPrice) external override onlyOwner {
        price = newPrice;
        emit PGC1Events.PriceUpdated(newPrice);
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
