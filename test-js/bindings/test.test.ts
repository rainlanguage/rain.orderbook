import assert from "assert";
import { describe, it } from "vitest";
import { getOrderHash, OrderV3 } from "../../cjs";

describe("Rain Orderbook Bindings Package Bindgen Tests", async function () {
  it("should get correct order hash", async () => {
    const order: OrderV3 = {
      owner: "0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba",
      evaluable: {
        interpreter: "0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba",
        store: "0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba",
        bytecode: Uint8Array.from([1, 2]),
      },
      validInputs: [
        {
          token: "0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba",
          decimals: 7,
          vaultId: "0",
        },
      ],
      validOutputs: [
        {
          token: "0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba",
          decimals: 18,
          vaultId: "0x1234",
        },
      ],
      nonce: "0x2",
    };
    const result = getOrderHash(order);
    const expected =
      "0xf4058d50e798f18a048097265fe67fe2e8619f337b9377a7620bb87fc2f52721";
    assert.equal(result, expected);
  });
});
