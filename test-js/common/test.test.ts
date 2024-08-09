import assert from "assert";
import { getLocal } from "mockttp";
import { getAddOrderCalldata } from "../../cjs";
import { describe, it, beforeEach, afterEach } from "vitest";

describe("Rain Orderbook Common Package Bindgen Tests", async function () {
  const mockServer = getLocal();
  beforeEach(() => mockServer.start(8080));
  afterEach(() => mockServer.stop());

  const dotrain = `
networks:
    some-network:
        rpc: http://localhost:8080/rpc-url
        chain-id: 123
        network-id: 123
        currency: ETH

subgraphs:
    some-sg: https://www.some-sg.com

deployers:
    some-deployer:
        network: some-network
        address: 0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba

orderbooks:
    some-orderbook:
        address: 0xc95A5f8eFe14d7a20BD2E5BAFEC4E71f8Ce0B9A6
        network: some-network
        subgraph: some-sg

tokens:
    token1:
        network: some-network
        address: 0xc2132d05d31c914a87c6611c10748aeb04b58e8f
        decimals: 6
        label: T1
        symbol: T1
    token2:
        network: some-network
        address: 0x8f3cf7ad23cd3cadbd9735aff958023239c6a063
        decimals: 18
        label: T2
        symbol: T2

scenarios:
    some-scenario:
        network: some-network
        deployer: some-deployer

orders:
    some-order:
      inputs:
        - token: token1
          vault-id: 1
      outputs:
        - token: token2
          vault-id: 1
      deployer: some-deployer
      orderbook: some-orderbook

deployments:
    some-deployment:
        scenario: some-scenario
        order: some-order
---
#calculate-io
_ _: 0 0;
#handle-io
:;
#post-add-order
:;
`;

  it("should get correct calldata", async () => {
    // mock calls
    // iInterpreter() call
    await mockServer
      .forPost("/rpc-url")
      .withBodyIncluding("0xf0cfdd37")
      .thenSendJsonRpcResult(`0x${"0".repeat(24) + "1".repeat(40)}`);
    // iStore() call
    await mockServer
      .forPost("/rpc-url")
      .withBodyIncluding("0xc19423bc")
      .thenSendJsonRpcResult(`0x${"0".repeat(24) + "2".repeat(40)}`);
    // iParser() call
    await mockServer
      .forPost("/rpc-url")
      .withBodyIncluding("0x24376855")
      .thenSendJsonRpcResult(`0x${"0".repeat(24) + "3".repeat(40)}`);
    // parse2() call
    await mockServer
      .forPost("/rpc-url")
      .withBodyIncluding("0xa3869e14")
      // 0x1234 encoded bytes
      .thenSendJsonRpcResult(
        "0x000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000021234000000000000000000000000000000000000000000000000000000000000"
      );

    const result = await getAddOrderCalldata(dotrain, "some-deployment");
    assert.equal(result.length, 1156);
  });

  it("should throw undefined deployment error", async () => {
    try {
      await getAddOrderCalldata(dotrain, "some-other-deployment");
      assert.fail("expected to fail, but resolved");
    } catch (error) {
      assert.ok(error instanceof Error);
      assert.equal(error.message, "undefined deployment");
    }
  });

  it("should throw frontmatter missing field error", async () => {
    try {
      const dotrain = `
deployers:
    some-deployer:
        ---
#calculate-io
_ _: 0 0;
#handle-io
:;
#post-add-order
:;
`;
      await getAddOrderCalldata(dotrain, "some-deployment");
      assert.fail("expected to fail, but resolved");
    } catch (error) {
      assert.ok(error instanceof Error);
      assert.equal(
        error.message,
        "deployers.some-deployer: missing field `address` at line 3 column 19"
      );
    }
  });
});
