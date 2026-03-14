// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {
    OrderBookV6ArbOrderTaker,
    TaskV2,
    OrderBookV6ArbConfig
} from "../../../src/abstract/OrderBookV6ArbOrderTaker.sol";
import {SignedContextV1} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {EvaluableV4} from "rain.interpreter.interface/interface/IInterpreterCallerV4.sol";
import {IInterpreterStoreV3} from "rain.interpreter.interface/interface/IInterpreterStoreV3.sol";
import {IInterpreterV4} from "rain.interpreter.interface/interface/IInterpreterV4.sol";

/// @dev We need a contract that is deployable in order to test the abstract
/// base contract.
contract ChildOrderBookV6ArbOrderTaker is OrderBookV6ArbOrderTaker {
    constructor()
        OrderBookV6ArbOrderTaker(OrderBookV6ArbConfig(
                TaskV2({
                    evaluable: EvaluableV4(IInterpreterV4(address(0)), IInterpreterStoreV3(address(0)), ""),
                    signedContext: new SignedContextV1[](0)
                })
            ))
    {}
}
