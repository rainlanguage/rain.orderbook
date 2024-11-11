import assert from "assert";
import { beforeAll, describe, expect, it } from "vitest";
import { DotrainOrderGui } from "../../dist/cjs/js_api.js";
import { Gui } from "../../dist/types/js_api.js";

const guiConfig = `
gui:
  name: Fixed limit
  description: Fixed limit order strategy
  deployments:
    - deployment: some-deployment
      name: Buy WETH with USDC on Base.
      description: Buy WETH with USDC for fixed price on Base network.
      deposits:
        - token: token1
          min: 0
          presets:
            - 0
            - 10
            - 100
            - 1000
            - 10000
      fields:
        - binding: binding-1
          name: Field 1 name
          description: Field 1 description
          presets:
            - name: Preset 1
              value:
                type: address
                value: "0x1234567890abcdef1234567890abcdef12345678"
            - name: Preset 2
              value:
                type: boolean
                value: false
            - name: Preset 3
              value:
                type: text
                value: "some-string"
        - binding: binding-2
          name: Field 2 name
          description: Field 2 description
          min: 100
          presets:
            - value:
                type: number
                value: 99.2
            - value:
                type: number
                value: 582.1
            - value:
                type: number
                value: 648.239
`;

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
#handle-add-order
:;
`;
const dotrainWithGui = `
${guiConfig}

${dotrain}
`;

describe("Rain Orderbook JS API Package Bindgen Tests - Gui", async function () {
  it("should return error if gui config is not found", async () => {
    await expect(
      DotrainOrderGui.init(dotrain, "some-deployment")
    ).rejects.toEqual(new Error("Gui config not found"));
  });

  it("should initialize gui object", async () => {
    const gui = await DotrainOrderGui.init(dotrainWithGui, "some-deployment");
    const guiConfig = gui.getGuiConfig() as Gui;
    assert.equal(guiConfig.name, "Fixed limit");
    assert.equal(guiConfig.description, "Fixed limit order strategy");
  });

  describe("deposit tests", async () => {
    let gui: DotrainOrderGui;
    beforeAll(async () => {
      gui = await DotrainOrderGui.init(dotrainWithGui, "some-deployment");
    });

    it("should add deposit", async () => {
      gui.saveDeposit({
        token: "token1",
        amount: 50.6,
        address: "0x1234567890abcdef1234567890abcdef12345678",
      });
      const deposits = gui.getDeposits();
      assert.equal(deposits.length, 1);
    });

    it("should remove deposit", async () => {
      gui.saveDeposit({
        token: "token1",
        amount: 50.6,
        address: "0x1234567890abcdef1234567890abcdef12345678",
      });
      const deposits = gui.getDeposits();
      assert.equal(deposits.length, 1);

      gui.removeDeposit("token1");
      const depositsAfterRemove = gui.getDeposits();
      assert.equal(depositsAfterRemove.length, 0);
    });
  });

  describe("field value tests", async () => {
    let gui: DotrainOrderGui;
    beforeAll(async () => {
      gui = await DotrainOrderGui.init(dotrainWithGui, "some-deployment");
    });

    it("should save field value", async () => {
      gui.saveFieldValues([
        {
          binding: "binding-1",
          value: {
            type: "address",
            value: "0x1234567890abcdef1234567890abcdef12345678",
          },
        },
      ]);
      gui.saveFieldValues([
        {
          binding: "binding-2",
          value: {
            type: "number",
            value: 100,
          },
        },
      ]);
      gui.saveFieldValues([
        {
          binding: "binding-1",
          value: {
            type: "text",
            value: "some-string",
          },
        },
      ]);
      gui.saveFieldValues([
        {
          binding: "binding-2",
          value: {
            type: "boolean",
            value: true,
          },
        },
      ]);
      const fieldValues = gui.getAllFieldValues();
      assert.equal(fieldValues.length, 2);
    });

    it("should get field value", async () => {
      gui.saveFieldValues([
        {
          binding: "binding-1",
          value: {
            type: "address",
            value: "0x1234567890abcdef1234567890abcdef12345678",
          },
        },
      ]);
      let fieldValue = gui.getFieldValue("binding-1");
      assert.equal(
        fieldValue.value,
        "0x1234567890abcdef1234567890abcdef12345678"
      );

      gui.saveFieldValues([
        {
          binding: "binding-2",
          value: {
            type: "boolean",
            value: true,
          },
        },
      ]);
      fieldValue = gui.getFieldValue("binding-2");
      assert.equal(fieldValue.value, true);

      gui.saveFieldValues([
        {
          binding: "binding-1",
          value: {
            type: "text",
            value: "some-string",
          },
        },
      ]);
      fieldValue = gui.getFieldValue("binding-1");
      assert.equal(fieldValue.value, "some-string");

      gui.saveFieldValues([
        {
          binding: "binding-2",
          value: {
            type: "number",
            value: 100.5,
          },
        },
      ]);
      fieldValue = gui.getFieldValue("binding-2");
      assert.equal(BigInt(fieldValue.value), BigInt("100500000000000000000"));
    });
  });

  describe("field definition tests", async () => {
    let gui: DotrainOrderGui;
    beforeAll(async () => {
      gui = await DotrainOrderGui.init(dotrainWithGui, "some-deployment");
    });

    it("should get field definition", async () => {
      const allFieldDefinitions = gui.getAllFieldDefinitions();
      assert.equal(allFieldDefinitions.length, 2);

      const fieldDefinition = gui.getFieldDefinition("binding-1");
      assert.equal(fieldDefinition.name, "Field 1 name");
      assert.equal(fieldDefinition.description, "Field 1 description");
      assert.equal(fieldDefinition.presets.length, 3);

      const preset1 = fieldDefinition.presets[0];
      assert.equal(preset1.name, "Preset 1");
      assert.equal(
        preset1.value.value,
        "0x1234567890abcdef1234567890abcdef12345678"
      );
      const preset2 = fieldDefinition.presets[1];
      assert.equal(preset2.name, "Preset 2");
      assert.equal(preset2.value.value, false);
      const preset3 = fieldDefinition.presets[2];
      assert.equal(preset3.name, "Preset 3");
      assert.equal(preset3.value.value, "some-string");

      const fieldDefinition2 = gui.getFieldDefinition("binding-2");
      assert.equal(
        BigInt(fieldDefinition2.presets[0].value.value),
        BigInt("99200000000000000000")
      );
      assert.equal(
        BigInt(fieldDefinition2.presets[1].value.value),
        BigInt("582100000000000000000")
      );
      assert.equal(
        BigInt(fieldDefinition2.presets[2].value.value),
        BigInt("648239000000000000000")
      );
    });
  });
});
