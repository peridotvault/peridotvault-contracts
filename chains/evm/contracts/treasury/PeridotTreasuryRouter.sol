// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {ReentrancyGuard} from "@openzeppelin/contracts/utils/ReentrancyGuard.sol";

contract PeridotTreasuryRouter is ReentrancyGuard {
    using SafeERC20 for IERC20;

    /// M5: contract schema version.
    uint8 public constant CONTRACT_VERSION = 1;

    // ---- Admin set (minimal) ----
    mapping(address => bool) public isAdmin;
    uint256 public adminCount;

    // ---- Recipient (the actual treasury destination) ----
    address public recipient;

    // ---- Events ----
    event AdminUpdated(address indexed admin, bool enabled);
    event RecipientUpdated(address indexed newRecipient);
    event SweptETH(address indexed to, uint256 amount);
    event SweptToken(address indexed token, address indexed to, uint256 amount);

    // ---- Errors (cheaper than revert strings) ----
    error NotAdmin();
    error ZeroAddress();
    error ZeroAmount();
    error LastAdmin();
    error EthSweepFailed();
    error InsufficientBalance();

    modifier onlyAdmin() {
        if (!isAdmin[msg.sender]) revert NotAdmin();
        _;
    }

    constructor(address initialRecipient, address[] memory initialAdmins) {
        if (initialRecipient == address(0)) revert ZeroAddress();
        recipient = initialRecipient;
        emit RecipientUpdated(initialRecipient);

        // Initialize admins
        // Requirement: at least 1 admin
        if (initialAdmins.length == 0) revert ZeroAddress();

        for (uint256 i = 0; i < initialAdmins.length; i++) {
            address a = initialAdmins[i];
            if (a == address(0)) revert ZeroAddress();
            if (!isAdmin[a]) {
                isAdmin[a] = true;
                adminCount++;
                emit AdminUpdated(a, true);
            }
        }
    }

    // -------------------------
    // Receiving funds
    // -------------------------

    /// @notice Accept ETH fees sent from PGC1 contracts.
    receive() external payable {}

    // -------------------------
    // Admin management
    // -------------------------

    /// @notice Add or remove an admin.
    /// @dev Prevent removing the last admin to avoid lock.
    function setAdmin(address admin, bool enabled) external onlyAdmin {
        if (admin == address(0)) revert ZeroAddress();

        bool current = isAdmin[admin];
        if (current == enabled) return; // no-op

        if (!enabled) {
            // Removing
            if (adminCount == 1) revert LastAdmin();
            isAdmin[admin] = false;
            adminCount--;
            emit AdminUpdated(admin, false);
        } else {
            // Adding
            isAdmin[admin] = true;
            adminCount++;
            emit AdminUpdated(admin, true);
        }
    }

    // -------------------------
    // Recipient management
    // -------------------------

    /// @notice Update the recipient treasury address.
    function setRecipient(address newRecipient) external onlyAdmin {
        if (newRecipient == address(0)) revert ZeroAddress();
        recipient = newRecipient;
        emit RecipientUpdated(newRecipient);
    }

    // -------------------------
    // Sweeping (pull-based)
    // -------------------------

    /// @notice Sweep all ETH held by this router to `recipient`.
    /// @dev M2 hardening: nonReentrant + checks-effects-interactions
    /// (balance read before the call), explicit ZeroAddress recipient gate,
    /// ZeroAmount and balance-sufficiency checks, event per withdrawal.
    function sweepETH() external onlyAdmin nonReentrant {
        if (recipient == address(0)) revert ZeroAddress();
        uint256 bal = address(this).balance;
        if (bal == 0) revert ZeroAmount();
        (bool ok, ) = recipient.call{value: bal}("");
        if (!ok) revert EthSweepFailed();
        emit SweptETH(recipient, bal);
    }

    /// @notice Sweep all balance of an ERC20 token held by this router to `recipient`.
    /// @dev M2 hardening: SafeERC20 (supports no-return USDT-style tokens),
    /// nonReentrant, zero-address gates, event per withdrawal.
    function sweepToken(IERC20 token) external onlyAdmin nonReentrant {
        if (address(token) == address(0)) revert ZeroAddress();
        if (recipient == address(0)) revert ZeroAddress();
        uint256 bal = token.balanceOf(address(this));
        if (bal == 0) revert ZeroAmount();
        token.safeTransfer(recipient, bal);
        emit SweptToken(address(token), recipient, bal);
    }
}
