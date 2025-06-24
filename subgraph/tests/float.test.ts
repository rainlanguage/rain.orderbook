import { createMockedFunction } from "matchstick-as";
import { Bytes, BigInt, ethereum } from "@graphprotocol/graph-ts";
import { FALLBACK_DECIMAL_FLOAT_ADDRESS } from "../src/float";

export const FLOAT_ZERO = Bytes.fromHexString(
  "0x0000000000000000000000000000000000000000000000000000000000000000"
);
export const FLOAT_1 = Bytes.fromHexString(
  "0x0000000000000000000000000000000000000000000000000000000000000001"
);
export const FLOAT_5 = Bytes.fromHexString(
  "0x0000000000000000000000000000000000000000000000000000000000000005"
);
export const FLOAT_10 = Bytes.fromHexString(
  "0x000000000000000000000000000000000000000000000000000000000000000a"
);
export const FLOAT_15 = Bytes.fromHexString(
  "0x000000000000000000000000000000000000000000000000000000000000000f"
);
export const FLOAT_20 = Bytes.fromHexString(
  "0x0000000000000000000000000000000000000000000000000000000000000014"
);
export const FLOAT_100 = Bytes.fromHexString(
  "0x0000000000000000000000000000000000000000000000000000000000000064"
);
export const FLOAT_200 = Bytes.fromHexString(
  "0x00000000000000000000000000000000000000000000000000000000000000c8"
);
export const FLOAT_300 = Bytes.fromHexString(
  "0x000000000000000000000000000000000000000000000000000000000000012c"
);
export const FLOAT_800 = Bytes.fromHexString(
  "0x0000000000000000000000000000000000000000000000000000000000000320"
);
export const FLOAT_900 = Bytes.fromHexString(
  "0x0000000000000000000000000000000000000000000000000000000000000384"
);
export const FLOAT_1000 = Bytes.fromHexString(
  "0x00000000000000000000000000000000000000000000000000000000000003e8"
);

export const FLOAT_NEG_10 = Bytes.fromHexString(
  "0x00000000fffffffffffffffffffffffffffffffffffffffffffffffffffffff6"
);
export const FLOAT_NEG_20 = Bytes.fromHexString(
  "0x00000000ffffffffffffffffffffffffffffffffffffffffffffffffffffffec"
);
export const FLOAT_NEG_100 = Bytes.fromHexString(
  "0x00000000ffffffffffffffffffffffffffffffffffffffffffffffffffffff9c"
);
export const FLOAT_NEG_200 = Bytes.fromHexString(
  "0x00000000ffffffffffffffffffffffffffffffffffffffffffffffffffffff38"
);

export function createMockDecimalFloatFunctions(): void {
  createMockedFunction(
    FALLBACK_DECIMAL_FLOAT_ADDRESS,
    "parse",
    "parse(string):(bytes4,bytes32)"
  )
    .withArgs([ethereum.Value.fromString("0")])
    .returns([
      ethereum.Value.fromFixedBytes(Bytes.fromHexString("0x00000000")),
      ethereum.Value.fromFixedBytes(FLOAT_ZERO),
    ]);

  function fromFixedDecimal(amount: BigInt, result: Bytes): void {
    createMockedFunction(
      FALLBACK_DECIMAL_FLOAT_ADDRESS,
      "fromFixedDecimalLosslessPacked",
      "fromFixedDecimalLosslessPacked(uint256,uint8):(bytes32)"
    )
      .withArgs([
        ethereum.Value.fromUnsignedBigInt(amount),
        ethereum.Value.fromUnsignedBigInt(BigInt.fromI32(18)),
      ])
      .returns([ethereum.Value.fromFixedBytes(result)]);
  }
  fromFixedDecimal(BigInt.fromI32(100), FLOAT_100);
  fromFixedDecimal(BigInt.fromI32(200), FLOAT_200);
  fromFixedDecimal(BigInt.fromI32(300), FLOAT_300);
  fromFixedDecimal(BigInt.fromI32(1000), FLOAT_1000);

  function minusMock(input: Bytes, output: Bytes): void {
    createMockedFunction(
      FALLBACK_DECIMAL_FLOAT_ADDRESS,
      "minus",
      "minus(bytes32):(bytes32)"
    )
      .withArgs([ethereum.Value.fromFixedBytes(input)])
      .returns([ethereum.Value.fromFixedBytes(output)]);
  }
  minusMock(FLOAT_100, FLOAT_NEG_100);
  minusMock(FLOAT_200, FLOAT_NEG_200);

  function addMock(a: Bytes, b: Bytes, sum: Bytes): void {
    createMockedFunction(
      FALLBACK_DECIMAL_FLOAT_ADDRESS,
      "add",
      "add(bytes32,bytes32):(bytes32)"
    )
      .withArgs([
        ethereum.Value.fromFixedBytes(a),
        ethereum.Value.fromFixedBytes(b),
      ])
      .returns([ethereum.Value.fromFixedBytes(sum)]);
  }
  addMock(FLOAT_ZERO, FLOAT_100, FLOAT_100);
  addMock(FLOAT_100, FLOAT_200, FLOAT_300);
  addMock(FLOAT_ZERO, FLOAT_300, FLOAT_300);
  addMock(FLOAT_ZERO, FLOAT_1000, FLOAT_1000);

  function subMock(a: Bytes, b: Bytes, diff: Bytes): void {
    createMockedFunction(
      FALLBACK_DECIMAL_FLOAT_ADDRESS,
      "sub",
      "sub(bytes32,bytes32):(bytes32)"
    )
      .withArgs([
        ethereum.Value.fromFixedBytes(a),
        ethereum.Value.fromFixedBytes(b),
      ])
      .returns([ethereum.Value.fromFixedBytes(diff)]);
  }
  subMock(FLOAT_1000, FLOAT_100, FLOAT_900); // 1000 - 100
  subMock(FLOAT_1000, FLOAT_200, FLOAT_800); // 1000 - 200
  subMock(FLOAT_300, FLOAT_200, FLOAT_100); // 300
  subMock(FLOAT_20, FLOAT_15, FLOAT_5); // 20 - 15 (clearing bounty)
  subMock(FLOAT_10, FLOAT_10, FLOAT_ZERO); // 10 - 10 = 0

  function gtMock(a: Bytes, b: Bytes, result: boolean): void {
    createMockedFunction(
      FALLBACK_DECIMAL_FLOAT_ADDRESS,
      "gt",
      "gt(bytes32,bytes32):(bool)"
    )
      .withArgs([
        ethereum.Value.fromFixedBytes(a),
        ethereum.Value.fromFixedBytes(b),
      ])
      .returns([ethereum.Value.fromBoolean(result)]);
  }
  gtMock(FLOAT_5, FLOAT_ZERO, true); // 5 > 0
  gtMock(FLOAT_ZERO, FLOAT_ZERO, false); // 0 > 0 ==> false
}
