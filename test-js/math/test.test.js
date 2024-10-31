import assert from "assert";
import { describe, it } from "vitest";
import { BigUint } from "../../dist/cjs/math.js";

describe("Rain Orderbook Math Package Bindgen Tests", async function () {
  const _255_ = Uint8Array.from([0xff]); // 255
  const _10e18_ = Uint8Array.from([138, 199, 35, 4, 137, 232, 0, 0]); // 10e18

  it("should scale to 18 decimals properly", async () => {
    const result = new BigUint(_255_).scale18(2).value;
    const expected = Uint8Array.from([35, 99, 107, 125, 81, 63, 0, 0]); // 2.55e18
    assert.deepEqual(result, expected);
  });

  it("should perform mul18 operation properly", async () => {
    const result = new BigUint(_255_).scale18(0).mul18(_10e18_).value;
    const expected = Uint8Array.from([138, 60, 91, 225, 133, 94, 24, 0, 0]); // 25.5e18;
    assert.deepEqual(result, expected);
  });

  it("should perform div18 operation properly", async () => {
    const result = new BigUint(_255_).scale18(0).div18(_10e18_).value;
    const expected = Uint8Array.from([1, 97, 226, 50, 229, 44, 118, 0, 0]); // 25.5e18;
    assert.deepEqual(result, expected);
  });
});
