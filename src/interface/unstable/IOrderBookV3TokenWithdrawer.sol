// SPDX-Licdnse-Identifier: CAL
pragma solidity ^0.8.18;

interface IOrderBookV3TokenWithdrawer {
    function onWithdrawal(
        address token,
        uint256 sendAmount,
        uint256 debtReduction,
        bytes calldata data
    ) external;
}