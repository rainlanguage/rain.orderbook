import {
  test,
  assert,
  createMockedFunction,
  clearStore,
  describe,
  afterEach,
  beforeAll,
  afterAll,
  newMockEvent,
  clearInBlockStore,
} from "matchstick-as";
import { Bytes, BigInt, ethereum, Address } from "@graphprotocol/graph-ts";

describe("noop", () => {
  test("noop", () => {
    let value = ethereum.Value.fromBytes(Bytes.fromHexString(""));
    assert.equals(value, value);
  });
});
