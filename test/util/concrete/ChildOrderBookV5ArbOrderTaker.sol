// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {
    OrderBookV5ArbOrderTaker,
    SignedContextV1,
    EvaluableV4,
    TaskV2,
    OrderBookV5ArbConfig
} from "src/abstract/OrderBookV5ArbOrderTaker.sol";
import {IInterpreterStoreV3} from "rain.interpreter.interface/interface/unstable/IInterpreterStoreV3.sol";
import {IInterpreterV4} from "rain.interpreter.interface/interface/unstable/IInterpreterV4.sol";

/// @dev We need a contract that is deployable in order to test the abstract
/// base contract.
contract ChildOrderBookV5ArbOrderTaker is OrderBookV5ArbOrderTaker {
    constructor()
        OrderBookV5ArbOrderTaker(
            OrderBookV5ArbConfig(
                address(0),
                TaskV2({
                    evaluable: EvaluableV4(IInterpreterV4(address(0)), IInterpreterStoreV3(address(0)), ""),
                    signedContext: new SignedContextV1[](0)
                }),
                abi.encode(address(0))
            )
        )
    {}
}
