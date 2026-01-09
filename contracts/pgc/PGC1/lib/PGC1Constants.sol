// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

library PGC1Constants {
    uint256 internal constant LICENSE_ID = 1;
    uint16 internal constant BPS_DENOMINATOR = 10_000;
    uint16 internal constant PLATFORM_FEE_BPS_CAP = 2_000; // 20% cap (adjustable policy)
}
