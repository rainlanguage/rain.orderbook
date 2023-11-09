// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {OrderBookExternalMockTest} from "test/util/abstract/OrderBookExternalMockTest.sol";
import {IERC3156FlashBorrower, ON_FLASH_LOAN_CALLBACK_SUCCESS} from "src/interface/ierc3156/IERC3156FlashBorrower.sol";
import {IERC20} from "lib/openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {FlashLenderCallbackFailed} from "src/abstract/OrderBookV3FlashLender.sol";

/// @title OrderBookV3FlashLenderGriefRecipientTest
/// Try to grief the recipient of the flash loan.
contract OrderBookClearTest is OrderBookExternalMockTest {
    /// Tests that no matter who the receiver is, and no matter what happens with
    /// the tokens, the flash loan will revert if the receiver is not
    /// `OrderBookV3FlashBorrower`.
    function testFlashLoanToNonReceiver(
        uint256 amount,
        bytes memory data,
        bytes32 notFlashLoanSuccess,
        bytes memory notFlashLoanSuccessBytes
    ) public {
        vm.assume(notFlashLoanSuccess != ON_FLASH_LOAN_CALLBACK_SUCCESS);
        vm.assume(keccak256(notFlashLoanSuccessBytes) != keccak256(abi.encode(ON_FLASH_LOAN_CALLBACK_SUCCESS)));

        // Return true for all transfers.
        vm.mockCall(address(iToken0), abi.encodeWithSelector(IERC20.transfer.selector), abi.encode(true));
        vm.mockCall(address(iToken0), abi.encodeWithSelector(IERC20.transferFrom.selector), abi.encode(true));

        // A call to an EOA will revert with no data.
        address receiver = address(0xDEADBEEF);
        vm.expectRevert();
        iOrderbook.flashLoan(IERC3156FlashBorrower(receiver), address(iToken0), amount, data);

        // A call to a contract that does not implement `IERC3156FlashBorrower`
        // will revert with `FlashLenderCallbackFailed`.
        vm.etch(receiver, hex"FE");
        vm.mockCall(
            receiver,
            abi.encodeWithSelector(IERC3156FlashBorrower.onFlashLoan.selector),
            abi.encode(notFlashLoanSuccess)
        );
        vm.expectRevert(abi.encodeWithSelector(FlashLenderCallbackFailed.selector, notFlashLoanSuccess));
        iOrderbook.flashLoan(IERC3156FlashBorrower(receiver), address(iToken0), amount, data);

        // A call to a contract that does not implement `IERC3156FlashBorrower`
        // will revert with no data if the return value is not `bytes32`.
        vm.mockCall(
            receiver, abi.encodeWithSelector(IERC3156FlashBorrower.onFlashLoan.selector), notFlashLoanSuccessBytes
        );
        vm.expectRevert();
        iOrderbook.flashLoan(IERC3156FlashBorrower(receiver), address(iToken0), amount, data);
    }

    /// Tests that if the receiver is some contract that returns
    /// `ON_FLASH_LOAN_CALLBACK_SUCCESS`, and the token movements do not error,
    /// then the flash loan will succeed.

}
