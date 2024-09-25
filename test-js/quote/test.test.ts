import assert from "assert";
import { getLocal } from "mockttp";
import { describe, it, beforeEach, afterEach } from "vitest";
import {
  QuoteSpec,
  QuoteTarget,
  OrderQuoteValue,
} from "../../dist/types/quote";
import {
  getId,
  doQuoteTargets,
  getQuoteTargetFromSubgraph,
} from "../../dist/cjs/quote";

describe("Rain Orderbook Quote Package Bindgen Tests", async function () {
  const mockServer = getLocal();
  beforeEach(() => mockServer.start(8081));
  afterEach(() => mockServer.stop());

  it("should get correct id", async () => {
    const orderbook = "0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba";
    const orderHash =
      "0xf4058d50e798f18a048097265fe67fe2e8619f337b9377a7620bb87fc2f52721";
    const result = getId(orderbook, orderHash);
    const expected =
      "0xca228cb816102ef9f8e0f9a87bb34e06c49c4d4ddf5a2a0ec229ab671475c235";
    assert.equal(result, expected);
  });

  it("should get correct quote targets from subgraph", async () => {
    await mockServer.forPost("/sg-url").thenReply(
      200,
      JSON.stringify({
        data: {
          orders: [
            {
              id: "0x46891c626a8a188610b902ee4a0ce8a7e81915e1b922584f8168d14525899dfb",
              orderBytes:
                "0x0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000012000000000000000000000000000000000000000000000000000000000000001a00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
              orderHash:
                "0x8a3fbb9caf53f18f1f78d90c48dbe4612bcd93285ed0fc033009b4a96ea2aaed",
              owner: "0x0000000000000000000000000000000000000000",
              outputs: [
                {
                  id: "0x0000000000000000000000000000000000000000",
                  token: {
                    id: "0x0000000000000000000000000000000000000000",
                    address: "0x0000000000000000000000000000000000000000",
                    name: "T1",
                    symbol: "T1",
                    decimals: "0",
                  },
                  balance: "0",
                  vaultId: "0",
                  owner: "0x0000000000000000000000000000000000000000",
                  ordersAsOutput: [],
                  ordersAsInput: [],
                  balanceChanges: [],
                  orderbook: {
                    id: "0x0000000000000000000000000000000000000000",
                  },
                },
              ],
              inputs: [
                {
                  id: "0x0000000000000000000000000000000000000000",
                  token: {
                    id: "0x0000000000000000000000000000000000000000",
                    address: "0x0000000000000000000000000000000000000000",
                    name: "T2",
                    symbol: "T2",
                    decimals: "0",
                  },
                  balance: "0",
                  vaultId: "0",
                  owner: "0x0000000000000000000000000000000000000000",
                  ordersAsOutput: [],
                  ordersAsInput: [],
                  balanceChanges: [],
                  orderbook: {
                    id: "0x0000000000000000000000000000000000000000",
                  },
                },
              ],
              active: true,
              addEvents: [
                {
                  transaction: {
                    blockNumber: "0",
                    timestamp: "0",
                    id: "0x0000000000000000000000000000000000000000",
                    from: "0x0000000000000000000000000000000000000000",
                  },
                },
              ],
              meta: null,
              timestampAdded: "0",
              orderbook: {
                id: "0x0000000000000000000000000000000000000000",
              },
              trades: [],
            },
          ],
        },
      })
    );

    const quoteSpec: QuoteSpec = {
      orderbook: "0x713180d188e1ff758f508d9f2e1d350d650fea5e",
      orderHash:
        "0x8a3fbb9caf53f18f1f78d90c48dbe4612bcd93285ed0fc033009b4a96ea2aaed",
      outputIOIndex: 0,
      inputIOIndex: 0,
      signedContext: [],
    };
    try {
      const result = await getQuoteTargetFromSubgraph(
        [quoteSpec],
        mockServer.url + "/sg-url"
      );
      const expected: QuoteTarget = {
        quoteConfig: {
          order: {
            owner: "0x0000000000000000000000000000000000000000",
            evaluable: {
              interpreter: "0x0000000000000000000000000000000000000000",
              store: "0x0000000000000000000000000000000000000000",
              bytecode: Uint8Array.from([]),
            },
            validInputs: [
              {
                token: "0x0000000000000000000000000000000000000000",
                decimals: 0,
                vaultId:
                  "0x0000000000000000000000000000000000000000000000000000000000000000",
              },
            ],
            validOutputs: [
              {
                token: "0x0000000000000000000000000000000000000000",
                decimals: 0,
                vaultId:
                  "0x0000000000000000000000000000000000000000000000000000000000000000",
              },
            ],
            nonce:
              "0x0000000000000000000000000000000000000000000000000000000000000000",
          },
          inputIOIndex: 0,
          outputIOIndex: 0,
          signedContext: [],
        },
        orderbook: "0x713180d188e1ff758f508d9f2e1d350d650fea5e",
      };
      assert.deepEqual(result[0], expected);
    } catch (error) {
      console.log(error);
      assert.fail("expected to resolve, but failed");
    }
  });

  it("should quote targets", async () => {
    await mockServer
      .forPost("/rpc-url")
      .thenSendJsonRpcResult(
        "0x000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002"
      );
    const target: QuoteTarget = {
      orderbook: "0xc6768d9e1cdd2f2058c92185364a3a5d2e1e47de",
      quoteConfig: {
        order: {
          owner: "0x0000000000000000000000000000000000000000",
          evaluable: {
            interpreter: "0x0000000000000000000000000000000000000000",
            store: "0x0000000000000000000000000000000000000000",
            bytecode: Uint8Array.from([]),
          },
          validInputs: [
            {
              token: "0x0000000000000000000000000000000000000000",
              decimals: 0,
              vaultId:
                "0x0000000000000000000000000000000000000000000000000000000000000000",
            },
          ],
          validOutputs: [
            {
              token: "0x0000000000000000000000000000000000000000",
              decimals: 0,
              vaultId:
                "0x0000000000000000000000000000000000000000000000000000000000000000",
            },
          ],
          nonce:
            "0x0000000000000000000000000000000000000000000000000000000000000000",
        },
        inputIOIndex: 0,
        outputIOIndex: 0,
        signedContext: [],
      },
    };
    try {
      const result = await doQuoteTargets(
        [target],
        mockServer.url + "/rpc-url"
      );
      const expected: OrderQuoteValue = {
        maxOutput:
          "0x0000000000000000000000000000000000000000000000000000000000000001",
        ratio:
          "0x0000000000000000000000000000000000000000000000000000000000000002",
      };
      assert.deepEqual(result[0], expected);
    } catch (error) {
      console.log(error);
      assert.fail("expected to resolve, but failed");
    }
  });
});
