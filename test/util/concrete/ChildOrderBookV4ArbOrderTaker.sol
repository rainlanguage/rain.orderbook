// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {
    OrderBookV4ArbOrderTaker,
    SignedContextV1,
    EvaluableV3,
    TaskV1,
    OrderBookV4ArbConfigV2
} from "src/abstract/OrderBookV4ArbOrderTaker.sol";
import {IInterpreterStoreV2} from "rain.interpreter.interface/interface/IInterpreterStoreV2.sol";
import {IInterpreterV3} from "rain.interpreter.interface/interface/IInterpreterV3.sol";

/// @dev We need a contract that is deployable in order to test the abstract
/// base contract.
contract ChildOrderBookV4ArbOrderTaker is OrderBookV4ArbOrderTaker {
    constructor()
        OrderBookV4ArbOrderTaker(
            OrderBookV4ArbConfigV2(
                address(0),
                TaskV1({
                    evaluable: EvaluableV3(IInterpreterV3(address(0)), IInterpreterStoreV2(address(0)), ""),
                    signedContext: new SignedContextV1[](0)
                }),
                abi.encode(address(0))
            )
        )
    {}
}
