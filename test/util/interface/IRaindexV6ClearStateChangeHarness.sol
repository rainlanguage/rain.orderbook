// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity ^0.8.25;

import {IRaindexV6, ClearStateChangeV2, Float} from "raindex-interface-0.1.8/src/interface/IRaindexV6.sol";
import {OrderIOCalculationV4} from "src/struct/OrderIOCalculationV4.sol";

interface IRaindexV6ClearStateChangeHarness is IRaindexV6 {
    function exposedCalculateClearStateChange(
        OrderIOCalculationV4 memory aliceOrderIOCalculation,
        OrderIOCalculationV4 memory bobOrderIOCalculation
    ) external pure returns (ClearStateChangeV2 memory);

    function exposedCalculateClearStateAlice(
        OrderIOCalculationV4 memory aliceOrderIOCalculation,
        OrderIOCalculationV4 memory bobOrderIOCalculation
    ) external pure returns (Float, Float);
}
