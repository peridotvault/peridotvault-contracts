// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import {ERC20} from "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import {
    ERC20Burnable
} from "@openzeppelin/contracts/token/ERC20/extensions/ERC20Burnable.sol";
import {
    ERC20Permit
} from "@openzeppelin/contracts/token/ERC20/extensions/ERC20Permit.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";

contract IDRX is ERC20, ERC20Burnable, ERC20Permit, Ownable {
    // ===== Faucet rules =====
    uint256 public constant MAX_MINT_AMOUNT = 50_000 * 1e18;
    uint256 public constant MINT_COOLDOWN = 1 days;

    mapping(address => uint256) public lastMintAt;

    // ===== Buy rules =====
    uint256 public constant RATE = 100_000_000;

    error MintCooldownActive();
    error MintAmountExceeded();
    error ZeroETH();
    error WithdrawFailed();

    constructor(
        address initialOwner
    ) ERC20("IDRX", "IDRX") ERC20Permit("IDRX") Ownable(initialOwner) {}

    /* =======================
            FAUCET MINT
       ======================= */
    function mint(uint256 amount) external {
        if (amount > MAX_MINT_AMOUNT) revert MintAmountExceeded();
        if (block.timestamp < lastMintAt[msg.sender] + MINT_COOLDOWN)
            revert MintCooldownActive();

        lastMintAt[msg.sender] = block.timestamp;
        _mint(msg.sender, amount);
    }

    /* =======================
            BUY WITH ETH
       ======================= */
    function buy() external payable {
        if (msg.value == 0) revert ZeroETH();
        _mint(msg.sender, msg.value * RATE);
    }

    /* =======================
            OWNER WITHDRAW
       ======================= */
    function withdrawETH() external onlyOwner {
        (bool ok, ) = owner().call{value: address(this).balance}("");
        if (!ok) revert WithdrawFailed();
    }

    /* =======================
            VIEW HELPERS
       ======================= */
    function previewBuy(uint256 ethAmountWei) external pure returns (uint256) {
        return ethAmountWei * RATE;
    }

    receive() external payable {}
}
