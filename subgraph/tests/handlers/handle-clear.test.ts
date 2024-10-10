import {
  test,
  assert,
  clearStore,
  describe,
  afterEach,
  clearInBlockStore,
} from "matchstick-as";
import { Bytes, BigInt, Address, log } from "@graphprotocol/graph-ts";
import {
  clearTemporaryDataEntityId,
  handleAfterClear,
  handleClear,
} from "../../src/clear";
import {
  AfterClearClearStateChangeStruct,
  ClearV2ClearConfigStruct,
  ClearV2Struct,
  createAfterClearEvent,
  createClearEvent,
  Evaluable,
  IO,
} from "../event-mocks.test";
import { createMockERC20Functions } from "../erc20.test";

describe("Clear", () => {
  afterEach(() => {
    clearStore();
    clearInBlockStore();
  });

  test("clearEvent and afterClearEvent", () => {
    let event = createClearEvent(
      Address.fromString("0x850c40aBf6e325231ba2DeD1356d1f2c267e63Ce"),
      new ClearV2Struct(
        Address.fromString("0x850c40aBf6e325231ba2DeD1356d1f2c267e63Ce"),
        new Evaluable(
          Address.fromString("0x5fB33D710F8B58DE4c9fDEC703B5c2487a5219d6"),
          Address.fromString("0x84c6e7F5A1e5dD89594Cc25BEf4722A1b8871aE6"),
          Bytes.fromHexString("0x1234567890123456789012345678901234567890")
        ),
        [
          new IO(
            Address.fromString("0x12e605bc104e93B45e1aD99F9e555f659051c2BB"),
            BigInt.fromI32(18),
            BigInt.fromI32(1)
          ),
        ],
        [
          new IO(
            Address.fromString("0x12e605bc104e93B45e1aD99F9e555f659051c2Bc"),
            BigInt.fromI32(18),
            BigInt.fromI32(1)
          ),
        ],
        Bytes.fromHexString(
          "0xbce73059f54ada335f7283df99f81d42a3f2d09527eade865627e26cd756b748"
        )
      ),
      new ClearV2Struct(
        Address.fromString("0x813aef302Ebad333EDdef619C6f8eD7FeF51BA7c"),
        new Evaluable(
          Address.fromString("0x5fB33D710F8B58DE4c9fDEC703B5c2487a5219d6"),
          Address.fromString("0x84c6e7F5A1e5dD89594Cc25BEf4722A1b8871aE6"),
          Bytes.fromHexString("0x1234567890123456789012345678901234567890")
        ),
        [
          new IO(
            Address.fromString("0x12e605bc104e93B45e1aD99F9e555f659051c2Bc"),
            BigInt.fromI32(18),
            BigInt.fromI32(2)
          ),
        ],
        [
          new IO(
            Address.fromString("0x12e605bc104e93B45e1aD99F9e555f659051c2Ba"),
            BigInt.fromI32(18),
            BigInt.fromI32(2)
          ),
          new IO(
            Address.fromString("0x12e605bc104e93B45e1aD99F9e555f659051c2BB"),
            BigInt.fromI32(18),
            BigInt.fromI32(2)
          ),
        ],
        Bytes.fromHexString(
          "0x9c8176f8e6e02b5f02eee226ff7066d2474bdc50f89bd15dca539240e0cb1788"
        )
      ),
      new ClearV2ClearConfigStruct(
        BigInt.fromI32(0),
        BigInt.fromI32(0),
        BigInt.fromI32(0),
        BigInt.fromI32(1),
        BigInt.fromI32(1),
        BigInt.fromI32(1)
      )
    );

    let aliceInput = event.params.alice.validInputs[0];
    assert.addressEquals(
      aliceInput.token,
      Address.fromString("0x12e605bc104e93B45e1aD99F9e555f659051c2BB")
    );
    let aliceOutput = event.params.alice.validOutputs[0];
    assert.addressEquals(
      aliceOutput.token,
      Address.fromString("0x12e605bc104e93B45e1aD99F9e555f659051c2Bc")
    );

    let bobInput = event.params.bob.validInputs[0];
    assert.addressEquals(
      bobInput.token,
      Address.fromString("0x12e605bc104e93B45e1aD99F9e555f659051c2Bc")
    );
    let bobOutput = event.params.bob.validOutputs[1];
    assert.addressEquals(
      bobOutput.token,
      Address.fromString("0x12e605bc104e93B45e1aD99F9e555f659051c2BB")
    );

    let id = clearTemporaryDataEntityId(event);
    handleClear(event);

    assert.entityCount("ClearTemporaryData", 1);
    assert.fieldEquals(
      "ClearTemporaryData",
      id.toHexString(),
      "aliceAddress",
      "0x850c40abf6e325231ba2ded1356d1f2c267e63ce"
    );
    assert.fieldEquals(
      "ClearTemporaryData",
      id.toHexString(),
      "bobAddress",
      "0x813aef302ebad333eddef619c6f8ed7fef51ba7c"
    );

    createMockERC20Functions(
      Address.fromString("0x12e605bc104e93B45e1aD99F9e555f659051c2BB")
    );
    createMockERC20Functions(
      Address.fromString("0x12e605bc104e93B45e1aD99F9e555f659051c2Bc")
    );
    createMockERC20Functions(
      Address.fromString("0x12e605bc104e93B45e1aD99F9e555f659051c2Ba")
    );

    let afterClearEvent = createAfterClearEvent(
      Address.fromString("0x850c40aBf6e325231ba2DeD1356d1f2c267e63Ce"),
      new AfterClearClearStateChangeStruct(
        BigInt.fromString("10000000000000000000"),
        BigInt.fromString("12476769284020210880"),
        BigInt.fromString("11308584431993808000"),
        BigInt.fromString("10000000000000000000")
      )
    );

    id = clearTemporaryDataEntityId(afterClearEvent);
    handleAfterClear(afterClearEvent);

    assert.entityCount("ClearTemporaryData", 0);
    assert.entityCount("Trade", 2);
  });
});
