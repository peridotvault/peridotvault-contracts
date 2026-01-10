// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

library PGC1Events {
    event Purchased(address indexed buyer, uint256 pricePaid);
    event PriceUpdated(uint256 newPrice);

    event MaxSupplyInitialized(uint256 maxSupply);
}
