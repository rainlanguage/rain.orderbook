import {
  test,
  assert,
  clearStore,
  describe,
  afterEach,
  newMockEvent,
  clearInBlockStore,
} from "matchstick-as";
import { createTransactionEntity } from "../src/transaction";

describe("Transaction", () => {
  afterEach(() => {
    clearStore();
    clearInBlockStore();
  });

  test("createTransactionEntity()", () => {
    let event = newMockEvent();

    createTransactionEntity(event);

    assert.entityCount("Transaction", 1);
    assert.fieldEquals(
      "Transaction",
      event.transaction.hash.toHex(),
      "blockNumber",
      event.block.number.toString()
    );
    assert.fieldEquals(
      "Transaction",
      event.transaction.hash.toHex(),
      "timestamp",
      event.block.timestamp.toString()
    );
    assert.fieldEquals(
      "Transaction",
      event.transaction.hash.toHex(),
      "from",
      event.transaction.from.toHex()
    );
  });

  test("has no problem with multiple events in the same transaction", () => {
    let event1 = newMockEvent();
    let event2 = newMockEvent();
    event2.transaction.hash = event1.transaction.hash;

    assert.bytesEquals(event1.transaction.hash, event2.transaction.hash);

    createTransactionEntity(event1);
    createTransactionEntity(event2);

    assert.entityCount("Transaction", 2);
    assert.fieldEquals(
      "Transaction",
      event1.transaction.hash.toHex(),
      "blockNumber",
      event1.block.number.toString()
    );
    assert.fieldEquals(
      "Transaction",
      event1.transaction.hash.toHex(),
      "timestamp",
      event1.block.timestamp.toString()
    );
    assert.fieldEquals(
      "Transaction",
      event1.transaction.hash.toHex(),
      "from",
      event1.transaction.from.toHex()
    );
    assert.fieldEquals(
      "Transaction",
      event2.transaction.hash.toHex(),
      "blockNumber",
      event2.block.number.toString()
    );
    assert.fieldEquals(
      "Transaction",
      event2.transaction.hash.toHex(),
      "timestamp",
      event2.block.timestamp.toString()
    );
    assert.fieldEquals(
      "Transaction",
      event2.transaction.hash.toHex(),
      "from",
      event2.transaction.from.toHex()
    );
  });
});
