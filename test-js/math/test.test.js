import assert from "assert";
import { describe, it } from "vitest";
import { BigUint } from "../../dist/cjs/math.js";

describe("Rain Orderbook Math Package Bindgen Tests", async function () {
  const tenE18AsUint8Array = Uint8Array.from([138, 199, 35, 4, 137, 232, 0, 0]); // 10e18
  it("should scale properly", async () => {
    const uint = new BigUint(Uint8Array.from([0xff]));
    const result = uint.scale18(2).value;
    const expected = Uint8Array.from([35, 99, 107, 125, 81, 63, 0, 0]); // 2.55e18
    assert.deepEqual(result, expected);
  });

  it("should mul18 properly", async () => {
    const result = new BigUint(Uint8Array.from([0xff]))
      .scale18(0)
      .mul18(tenE18AsUint8Array).value;
    const expected = Uint8Array.from([138, 60, 91, 225, 133, 94, 24, 0, 0]); // 25.5e18;
    assert.deepEqual(result, expected);
  });

  it("should div18 properly", async () => {
    const result = new BigUint(Uint8Array.from([0xff]))
      .scale18(0)
      .div18(tenE18AsUint8Array).value;
    const expected = Uint8Array.from([1, 97, 226, 50, 229, 44, 118, 0, 0]); // 25.5e18;
    assert.deepEqual(result, expected);
  });
});
