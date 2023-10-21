// SPDX-License-Identifier: CAL
pragma solidity ^0.8.18;

string constant ORDER_BOOK_META_PATH = "meta/OrderBook.rain.meta";

abstract contract OrderBookTestMetaPath {
    function orderBookMetaPath() internal virtual returns (string memory) {
        return ORDER_BOOK_META_PATH;
    }
}