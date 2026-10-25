// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import {XChainEnvelope} from "../registry/XChainEnvelope.sol";

/// @notice Test harness exposing the XChainEnvelope dest rule for AC-ENV tests.
contract XChainEnvelopeHarness {
    function verify(XChainEnvelope.Envelope memory e) external pure {
        XChainEnvelope.verifyForDest(e);
    }
}
