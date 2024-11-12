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
            - "0"
            - "10"
            - "100"
            - "1000"
            - "10000"
      fields:
        - binding: binding-1
          name: Field 1 name
          description: Field 1 description
          presets:
            - name: Preset 1
              value: "0x1234567890abcdef1234567890abcdef12345678"
            - name: Preset 2
              value: "false"
            - name: Preset 3
              value: "some-string"
        - binding: binding-2
          name: Field 2 name
          description: Field 2 description
          min: 100
          presets:
            - value: "99.2"
            - value: "582.1"
            - value: "648.239"
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
      gui.saveDeposit("token1", "50.6");
      const deposits = gui.getDeposits();
      assert.equal(deposits.length, 1);
    });

    it("should throw error if deposit token is not found in gui config", () => {
      expect(() => gui.saveDeposit("token3", "1")).toThrow(
        "Deposit token not found in gui config: token3"
      );
    });

    it("should remove deposit", async () => {
      gui.saveDeposit("token1", "50.6");
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
          value: "0x1234567890abcdef1234567890abcdef12345678",
        },
        {
          binding: "binding-2",
          value: "100",
        },
      ]);
      gui.saveFieldValues([
        {
          binding: "binding-1",
          value: "some-string",
        },
        {
          binding: "binding-2",
          value: "true",
        },
      ]);
      const fieldValues = gui.getAllFieldValues();
      assert.equal(fieldValues.length, 2);
    });

    it("should throw error during save if field binding is not found in field definitions", () => {
      expect(() => gui.saveFieldValue("binding-3", "1")).toThrow(
        "Field binding not found: binding-3"
      );
    });

    it("should get field value", async () => {
      gui.saveFieldValue(
        "binding-1",
        "0x1234567890abcdef1234567890abcdef12345678"
      );
      let fieldValue = gui.getFieldValue("binding-1");
      assert.equal(fieldValue, "0x1234567890abcdef1234567890abcdef12345678");

      gui.saveFieldValue("binding-2", "true");
      fieldValue = gui.getFieldValue("binding-2");
      assert.equal(fieldValue, "true");

      gui.saveFieldValue("binding-1", "some-string");
      fieldValue = gui.getFieldValue("binding-1");
      assert.equal(fieldValue, "some-string");

      gui.saveFieldValue("binding-2", "100.5");
      fieldValue = gui.getFieldValue("binding-2");
      assert.equal(fieldValue, "100.5");
    });

    it("should throw error during get if field binding is not found", () => {
      expect(() => gui.getFieldValue("binding-3")).toThrow(
        "Field binding not found: binding-3"
      );
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
      assert.equal(preset1.value, "0x1234567890abcdef1234567890abcdef12345678");
      const preset2 = fieldDefinition.presets[1];
      assert.equal(preset2.name, "Preset 2");
      assert.equal(preset2.value, "false");
      const preset3 = fieldDefinition.presets[2];
      assert.equal(preset3.name, "Preset 3");
      assert.equal(preset3.value, "some-string");

      const fieldDefinition2 = gui.getFieldDefinition("binding-2");
      assert.equal(fieldDefinition2.presets[0].value, "99.2");
      assert.equal(fieldDefinition2.presets[1].value, "582.1");
      assert.equal(fieldDefinition2.presets[2].value, "648.239");
    });

    it("should throw error during get if field binding is not found", () => {
      expect(() => gui.getFieldDefinition("binding-3")).toThrow(
        "Field binding not found: binding-3"
      );
    });
  });

  describe("state management tests", async () => {
    let serializedString =
      "H4sIAAAAAAAA_3WNwQ6CMBBE_2XPaHahLYVfMca03cU0YjEWjAnh360cvHmazEzezApDlJEvLzcukqFfwcfEMV0PBD3gm-pGadPaDp0PLMM_D9UPrAtIiLBVwPKYcpzL7mmFebpJKt2uVAB3n5Y0l0Tj0Xw981Ny3n9DTU3NqLmh0JFytg3GEAXCVlknHpXXVuwA23n7AOb4sTzEAAAA";
    let gui: DotrainOrderGui;
    beforeAll(async () => {
      gui = await DotrainOrderGui.init(dotrainWithGui, "some-deployment");

      gui.saveFieldValue(
        "binding-1",
        "0x1234567890abcdef1234567890abcdef12345678"
      );
      gui.saveFieldValue("binding-2", "100");
      gui.saveDeposit("token1", "50.6");
    });

    it("should serialize gui state", async () => {
      const serialized = gui.serializeState();
      assert.equal(serialized, serializedString);
    });

    it("should deserialize gui state", async () => {
      gui.clearState();
      gui.deserializeState(serializedString);
      const fieldValues = gui.getAllFieldValues();
      assert.equal(fieldValues.length, 2);
      assert.equal(fieldValues[0].binding, "binding-1");
      assert.equal(
        fieldValues[0].value,
        "0x1234567890abcdef1234567890abcdef12345678"
      );
      assert.equal(fieldValues[1].binding, "binding-2");
      assert.equal(fieldValues[1].value, "100");
      const deposits = gui.getDeposits();
      assert.equal(deposits.length, 1);
      assert.equal(deposits[0].token, "token1");
      assert.equal(deposits[0].amount, "50.6");
      assert.equal(
        deposits[0].address,
        "0xc2132d05d31c914a87c6611c10748aeb04b58e8f"
      );
    });

    it("should clear state", async () => {
      gui.clearState();
      const fieldValues = gui.getAllFieldValues();
      assert.equal(fieldValues.length, 0);
      const deposits = gui.getDeposits();
      assert.equal(deposits.length, 0);
    });
  });
});
