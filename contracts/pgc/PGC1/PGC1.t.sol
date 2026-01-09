// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import {PGC1} from "./PGC1.sol";
import {PGC1Constants} from "./lib/PGC1Constants.sol";
import {PGC1Errors} from "./lib/PGC1Errors.sol";

contract PGC1Test is Test {
    PGC1 pgc;

    address owner = address(this); // default msg.sender in tests
    address buyer = address(0xB0B);
    address treasury = address(0xBEEF);
    address dev = address(0xDEAD);

    // config
    string tokenURI1155 = "ipfs://base/{id}.json";
    bytes32 gameId = keccak256("peridot:studio:my-game");

    bytes32 contractMetaHashV1 = keccak256(bytes("contract-meta-v1"));
    string contractMetaUriV1 = "ipfs://contract/v1.json";

    uint256 price = 0.01 ether;
    uint256 maxSupply = 2;
    uint16 feeBps = 1_000; // 10%

    function setUp() public {
        // fund buyer so it can pay
        vm.deal(buyer, 100 ether);

        pgc = new PGC1(
            tokenURI1155,
            contractMetaHashV1,
            contractMetaUriV1,
            gameId,
            address(0), // ETH payment
            price,
            maxSupply,
            treasury,
            dev,
            feeBps
        );
    }

    // -------------------------
    // Constructor checks
    // -------------------------

    function test_constructor_setsInitialContractMetaCommit() public view {
        require(
            pgc.contractMetaHeadVersion() == 1,
            "contract meta version should be 1"
        );
        require(
            pgc.contractMetaHeadHash() == contractMetaHashV1,
            "contract meta hash mismatch"
        );
        require(
            pgc.contractMetaHeadParentHash() == bytes32(0),
            "contract meta parent should be 0"
        );
    }

    // -------------------------
    // Metadata publish + gating
    // -------------------------

    function test_buy_revert_ifNoMetadataPublished() public {
        vm.prank(buyer);
        vm.expectRevert(PGC1Errors.NoMetadataPublished.selector);
        pgc.buy{value: price}();
    }

    function test_publishMetadata_updatesHead() public {
        bytes32 h1 = keccak256("meta-v1");
        string memory u1 = "ipfs://game/meta-v1.json";

        pgc.publishMetadata(h1, u1);

        require(pgc.metadataHeadVersion() == 1, "meta version should be 1");
        require(pgc.metadataHeadHash() == h1, "meta hash mismatch");
        require(
            pgc.metadataHeadParentHash() == bytes32(0),
            "meta parent should be 0"
        );
    }

    function test_publishMetadata_onlyOwner() public {
        vm.prank(buyer);
        vm.expectRevert(); // Ownable revert string differs by OZ version
        pgc.publishMetadata(keccak256("x"), "ipfs://x.json");
    }

    // -------------------------
    // Buy flow (ETH)
    // -------------------------

    function test_buy_success_mints_and_splitsETH() public {
        // publish metadata first (required)
        pgc.publishMetadata(keccak256("meta-v1"), "ipfs://game/meta-v1.json");

        uint256 treasuryBefore = treasury.balance;
        uint256 devBefore = dev.balance;

        // expected split
        uint256 fee = (price * feeBps) / PGC1Constants.BPS_DENOMINATOR;
        uint256 devAmt = price - fee;

        vm.prank(buyer);
        pgc.buy{value: price}();

        require(
            pgc.balanceOf(buyer, PGC1Constants.LICENSE_ID) == 1,
            "buyer should own license"
        );
        require(
            pgc.totalSupply(PGC1Constants.LICENSE_ID) == 1,
            "totalSupply should be 1"
        );

        require(
            treasury.balance == treasuryBefore + fee,
            "treasury not paid correctly"
        );
        require(dev.balance == devBefore + devAmt, "dev not paid correctly");
    }

    function test_buy_revert_invalidPayment() public {
        pgc.publishMetadata(keccak256("meta-v1"), "ipfs://game/meta-v1.json");

        vm.prank(buyer);
        vm.expectRevert(PGC1Errors.InvalidPayment.selector);
        pgc.buy{value: price - 1}();
    }

    function test_buy_revert_alreadyOwned() public {
        pgc.publishMetadata(keccak256("meta-v1"), "ipfs://game/meta-v1.json");

        vm.prank(buyer);
        pgc.buy{value: price}();

        vm.prank(buyer);
        vm.expectRevert(PGC1Errors.AlreadyOwned.selector);
        pgc.buy{value: price}();
    }

    function test_buy_revert_soldOut() public {
        pgc.publishMetadata(keccak256("meta-v1"), "ipfs://game/meta-v1.json");

        address buyer2 = address(0xB0B2);
        address buyer3 = address(0xB0B3);
        vm.deal(buyer2, 100 ether);
        vm.deal(buyer3, 100 ether);

        vm.prank(buyer);
        pgc.buy{value: price}();

        vm.prank(buyer2);
        pgc.buy{value: price}();

        vm.prank(buyer3);
        vm.expectRevert(PGC1Errors.SoldOut.selector);
        pgc.buy{value: price}();
    }

    // -------------------------
    // Burn
    // -------------------------

    function test_burn_works() public {
        pgc.publishMetadata(keccak256("meta-v1"), "ipfs://game/meta-v1.json");

        vm.prank(buyer);
        pgc.buy{value: price}();

        vm.prank(buyer);
        pgc.burn();

        require(
            pgc.balanceOf(buyer, PGC1Constants.LICENSE_ID) == 0,
            "burn should clear balance"
        );
        require(
            pgc.totalSupply(PGC1Constants.LICENSE_ID) == 0,
            "burn should decrease supply"
        );
    }

    // -------------------------
    // Soulbound: transfers revert
    // -------------------------

    function test_soulbound_transferReverts() public {
        pgc.publishMetadata(keccak256("meta-v1"), "ipfs://game/meta-v1.json");

        vm.prank(buyer);
        pgc.buy{value: price}();

        address receiver = address(0xCAFE);

        vm.prank(buyer);
        vm.expectRevert(PGC1Errors.NonTransferable.selector);
        pgc.safeTransferFrom(buyer, receiver, PGC1Constants.LICENSE_ID, 1, "");
    }
}
