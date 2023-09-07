// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import "src/interface/unstable/IOrderBookV3.sol";

contract FlashLendingMockOrderBook is IOrderBookV3 {
    function flashLoan(IERC3156FlashBorrower receiver, address token, uint256 amount, bytes calldata data)
        external
        returns (bool)
    {
        receiver.onFlashLoan(msg.sender, token, amount, 0, data);
        return true;
    }

    function takeOrders(TakeOrdersConfigV2 calldata) external pure returns (uint256 totalInput, uint256 totalOutput) {
        return (0, 0);
    }

    function addOrder(OrderConfigV2 calldata) external pure returns (bool stateChanged) {
        return false;
    }

    function orderExists(bytes32) external pure returns (bool exists) {
        return false;
    }

    function clear(
        Order memory alice,
        Order memory bob,
        ClearConfig calldata clearConfig,
        SignedContextV1[] memory aliceSignedContextV1,
        SignedContextV1[] memory bobSignedContextV1
    ) external {}
    function deposit(address token, uint256 vaultId, uint256 amount) external {}
    function flashFee(address token, uint256 amount) external view returns (uint256) {}
    function maxFlashLoan(address token) external view returns (uint256) {}
    function removeOrder(Order calldata order) external returns (bool stateChanged) {}

    function vaultBalance(address owner, address token, uint256 id) external view returns (uint256 balance) {}
    function withdraw(address token, uint256 vaultId, uint256 targetAmount) external {}
}