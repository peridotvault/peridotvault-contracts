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

contract PGC1Factory is Ownable {
    using Clones for address;
    using SafeERC20 for IERC20;

    // -------------------------
    // Config
    // -------------------------
    address public immutable pgc1Implementation;

    IPeridotRegistry public registry;

    address public feeRecipient;

    /// @dev publish fee token. address(0) = ETH, otherwise ERC20 token.
    address public feeToken;

    /// @dev publish fee amount: wei (ETH) or smallest-unit (ERC20)
    uint256 public publishFee;

    // allowlist control
    bool public allowlistEnabled = true;
    mapping(address => bool) public isPublisher;

    // -------------------------
    // Events
    // -------------------------
    event RegistrySet(address indexed registry);

    event PublisherSet(address indexed publisher, bool allowed);
    event AllowlistEnabledSet(bool enabled);

    event PublishFeeSet(uint256 newFee);
    event FeeRecipientSet(address indexed newRecipient);
    event FeeTokenSet(address indexed newToken);

    event GamePublished(
        address indexed publisher,
        address indexed pgc1,
        bytes32 indexed gameId
    );

    // -------------------------
    // Init struct
    // -------------------------
    struct PGC1Init {
        string tokenURI1155;
        bytes32 initialContractMetaHash;
        string initialContractMetaURI;
        bytes32 gameId;
        address paymentToken;
        uint256 price;
        uint256 maxSupply;
        address treasuryRouter;
        address developerRecipient;
        uint16 platformFeeBps;
    }

    // -------------------------
    // Errors
    // -------------------------
    error NotAllowedPublisher();
    error EthNotAccepted(); // when feeToken is ERC20
    error RegistryNotSet();

    // -------------------------
    // Constructor
    // -------------------------
    constructor(
        address pgc1Implementation_,
        address feeRecipient_,
        address feeToken_, // address(0)=ETH; else ERC20
        uint256 publishFee_
    ) Ownable(msg.sender) {
        if (pgc1Implementation_ == address(0)) revert PGC1Errors.ZeroAddress();
        if (feeRecipient_ == address(0)) revert PGC1Errors.ZeroAddress();

        pgc1Implementation = pgc1Implementation_;
        feeRecipient = feeRecipient_;
        feeToken = feeToken_;
        publishFee = publishFee_;

        emit FeeRecipientSet(feeRecipient_);
        emit FeeTokenSet(feeToken_);
        emit PublishFeeSet(publishFee_);
    }

    // -------------------------
    // Admin: registry config
    // -------------------------
    function setRegistry(address registry_) external onlyOwner {
        if (registry_ == address(0)) revert PGC1Errors.ZeroAddress();
        registry = IPeridotRegistry(registry_);
        emit RegistrySet(registry_);
    }

    // -------------------------
    // Admin: allowlist
    // -------------------------
    function setAllowlistEnabled(bool enabled) external onlyOwner {
        allowlistEnabled = enabled;
        emit AllowlistEnabledSet(enabled);
    }

    function setPublisher(address publisher, bool allowed) external onlyOwner {
        if (publisher == address(0)) revert PGC1Errors.ZeroAddress();
        isPublisher[publisher] = allowed;
        emit PublisherSet(publisher, allowed);
    }

    function setPublishers(
        address[] calldata publishers,
        bool allowed
    ) external onlyOwner {
        for (uint256 i = 0; i < publishers.length; i++) {
            address p = publishers[i];
            if (p == address(0)) revert PGC1Errors.ZeroAddress();
            isPublisher[p] = allowed;
            emit PublisherSet(p, allowed);
        }
    }

    // -------------------------
    // Admin: fee config
    // -------------------------
    function setPublishFee(uint256 newFee) external onlyOwner {
        publishFee = newFee;
        emit PublishFeeSet(newFee);
    }

    function setFeeRecipient(address newRecipient) external onlyOwner {
        if (newRecipient == address(0)) revert PGC1Errors.ZeroAddress();
        feeRecipient = newRecipient;
        emit FeeRecipientSet(newRecipient);
    }

    /// @dev set token used to pay publish fee. address(0)=ETH; else ERC20.
    function setFeeToken(address newToken) external onlyOwner {
        feeToken = newToken;
        emit FeeTokenSet(newToken);
    }

    // -------------------------
    // Publish
    // -------------------------
    function publishGame(
        PGC1Init calldata init
    ) external payable returns (address pgc1) {
        if (allowlistEnabled && !isPublisher[msg.sender])
            revert NotAllowedPublisher();
        if (address(registry) == address(0)) revert RegistryNotSet();

        _collectPublishFee();

        // Deploy clone
        pgc1 = pgc1Implementation.clone();

        // Initialize clone; owner = msg.sender (publisher)
        PGC1(pgc1).initialize(
            init.tokenURI1155,
            init.initialContractMetaHash,
            init.initialContractMetaURI,
            init.gameId,
            init.paymentToken,
            init.price,
            init.maxSupply,
            init.treasuryRouter,
            init.developerRecipient,
            init.platformFeeBps,
            msg.sender
        );

        // Register in on-chain registry (atomic)
        registry.registerGame(init.gameId, pgc1, msg.sender);

        emit GamePublished(msg.sender, pgc1, init.gameId);
    }

    function _collectPublishFee() internal {
        uint256 fee = publishFee;

        if (feeToken == address(0)) {
            // ETH mode
            if (msg.value != fee) revert PGC1Errors.InvalidPayment();

            (bool ok, ) = payable(feeRecipient).call{value: msg.value}("");
            if (!ok) revert PGC1Errors.PayoutFailed();
        } else {
            // ERC20 mode
            if (msg.value != 0) revert EthNotAccepted();

            IERC20(feeToken).safeTransferFrom(msg.sender, feeRecipient, fee);
        }
    }
}
