import assert from 'assert';
import { afterAll, beforeAll, beforeEach, describe, expect, it } from 'vitest';
import { DotrainOrderGui } from '../../dist/cjs/js_api.js';
import {
	AddOrderCalldataResult,
	AllFieldValuesResult,
	AllowancesResult,
	ApprovalCalldataResult,
	AvailableDeployments,
	Config,
	DepositAndAddOrderCalldataResult,
	DepositCalldataResult,
	Gui,
	GuiDeployment,
	SelectTokens,
	TokenDeposit,
	TokenInfos
} from '../../dist/types/js_api.js';
import { getLocal } from 'mockttp';

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
    - deployment: other-deployment
      name: Test test
      description: Test test test
      deposits:
        - token: token1
          min: 0
          presets:
            - "0"
      fields:
        - binding: binding-1
          name: Field 1 name
          description: Field 1 description
          presets:
            - name: Preset 1
              value: "0"
        - binding: binding-2
          name: Field 2 name
          description: Field 2 description
          min: 100
          presets:
            - value: "0"
`;
const guiConfig2 = `
gui:
  name: Test test
  description: Test test test
  deployments:
    - deployment: other-deployment
      name: Test test
      description: Test test test
      deposits:
        - token: token1
          min: 0
          presets:
            - "0"
        - token: token2
          min: 0
          presets:
            - "0"
      fields:
        - binding: test-binding
          name: Test binding
          description: Test binding description
          presets:
            - value: "0xbeef"
`;
const guiConfig3 = `
gui:
  name: Test test
  description: Test test test
  deployments:
    - deployment: other-deployment
      name: Test test
      description: Test test test
      deposits:
        - token: token1
          min: 0
          presets:
            - "0"
        - token: token2
          min: 0
          presets:
            - "0"
      fields:
        - binding: test-binding
          name: Test binding
          description: Test binding description
          presets:
            - value: "test-value"
      select-tokens:
        - token1
        - token2
`;

const dotrain = `
networks:
    some-network:
        rpc: http://localhost:8085/rpc-url
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
        label: Token 1
        symbol: T1
    token2:
        network: some-network
        address: 0x8f3cf7ad23cd3cadbd9735aff958023239c6a063
        decimals: 18
        label: Token 2
        symbol: T2

scenarios:
    some-scenario:
        network: some-network
        deployer: some-deployer
        bindings:
            test-binding: "5"

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
    other-deployment:
        scenario: some-scenario
        order: some-order
---
#test-binding !
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
:;
`;
const dotrainWithoutVaultIds = `
networks:
    some-network:
        rpc: http://localhost:8085/rpc-url
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
        label: Token 1
        symbol: T1
    token2:
        network: some-network
        address: 0x8f3cf7ad23cd3cadbd9735aff958023239c6a063
        decimals: 18
        label: Token 2
        symbol: T2

scenarios:
    some-scenario:
        network: some-network
        deployer: some-deployer

orders:
    some-order:
      inputs:
        - token: token1
      outputs:
        - token: token2
      deployer: some-deployer
      orderbook: some-orderbook

deployments:
    some-deployment:
        scenario: some-scenario
        order: some-order
    other-deployment:
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

describe('Rain Orderbook JS API Package Bindgen Tests - Gui', async function () {
	const mockServer = getLocal();
	beforeAll(async () => {
		await mockServer.start(8085);
	});
	afterAll(async () => {
		await mockServer.stop();
	});
	beforeEach(() => {
		mockServer.reset();
	});

	it('should return available deployments', async () => {
		const deployments: AvailableDeployments =
			await DotrainOrderGui.getAvailableDeployments(dotrainWithGui);
		assert.equal(deployments.length, 2);
		assert.equal(deployments[0].deployment_name, 'some-deployment');
		assert.equal(deployments[1].deployment_name, 'other-deployment');
	});

	it('should return error if gui config is not found', async () => {
		await expect(DotrainOrderGui.chooseDeployment(dotrain, 'some-deployment')).rejects.toEqual(
			new Error('Gui config not found')
		);
	});

	it('should initialize gui object', async () => {
		// mock the rpc call to get token info
		mockServer
			.forPost('/rpc-url')
			.withBodyIncluding('0x82ad56cb')
			.thenSendJsonRpcResult(
				'0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000007546f6b656e203100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000025431000000000000000000000000000000000000000000000000000000000000'
			);

		const gui = await DotrainOrderGui.chooseDeployment(dotrainWithGui, 'some-deployment');
		const guiConfig = gui.getGuiConfig() as Gui;
		assert.equal(guiConfig.name, 'Fixed limit');
		assert.equal(guiConfig.description, 'Fixed limit order strategy');
	});

	it('should get token infos', async () => {
		mockServer
			.forPost('/rpc-url')
			.once()
			.withBodyIncluding('0x82ad56cb')
			.thenSendJsonRpcResult(
				'0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000007546f6b656e203100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000025431000000000000000000000000000000000000000000000000000000000000'
			);
		mockServer
			.forPost('/rpc-url')
			.once()
			.withBodyIncluding('0x82ad56cb')
			.thenSendJsonRpcResult(
				'0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000001200000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000007546f6b656e203200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000025432000000000000000000000000000000000000000000000000000000000000'
			);
		const dotrainWithGui = `
    ${guiConfig2}

    ${dotrain}
    `;
		const gui = await DotrainOrderGui.chooseDeployment(dotrainWithGui, 'other-deployment');

		const tokenInfos: TokenInfos = gui.getTokenInfos();
		const token1Address = '0xc2132d05d31c914a87c6611c10748aeb04b58e8f';
		const token2Address = '0x8f3cf7ad23cd3cadbd9735aff958023239c6a063';
		assert.equal(tokenInfos.get(token1Address)?.decimals, 6);
		assert.equal(tokenInfos.get(token1Address)?.name, 'Token 1');
		assert.equal(tokenInfos.get(token1Address)?.symbol, 'T1');
		assert.equal(tokenInfos.get(token2Address)?.decimals, 18);
		assert.equal(tokenInfos.get(token2Address)?.name, 'Token 2');
		assert.equal(tokenInfos.get(token2Address)?.symbol, 'T2');
	});

	describe('deposit tests', async () => {
		let gui: DotrainOrderGui;
		beforeAll(async () => {
			mockServer
				.forPost('/rpc-url')
				.withBodyIncluding('0x82ad56cb')
				.thenSendJsonRpcResult(
					'0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000007546f6b656e203100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000025431000000000000000000000000000000000000000000000000000000000000'
				);
			gui = await DotrainOrderGui.chooseDeployment(dotrainWithGui, 'some-deployment');
		});

		it('should add deposit', async () => {
			gui.saveDeposit('token1', '50.6');
			const deposits: TokenDeposit[] = gui.getDeposits();
			assert.equal(deposits.length, 1);
		});

		it('should update deposit', async () => {
			gui.saveDeposit('token1', '50.6');
			gui.saveDeposit('token1', '100.6');
			const deposits: TokenDeposit[] = gui.getDeposits();
			assert.equal(deposits.length, 1);
			assert.equal(deposits[0].amount, '100.6');
		});

		it('should throw error if deposit token is not found in gui config', () => {
			expect(() => gui.saveDeposit('token3', '1')).toThrow(
				'Deposit token not found in gui config: token3'
			);
		});

		it('should remove deposit', async () => {
			gui.saveDeposit('token1', '50.6');
			const deposits: TokenDeposit[] = gui.getDeposits();
			assert.equal(deposits.length, 1);

			gui.removeDeposit('token1');
			const depositsAfterRemove: TokenDeposit[] = gui.getDeposits();
			assert.equal(depositsAfterRemove.length, 0);
		});

		it('should get deposit presets', async () => {
			const presets = gui.getDepositPresets('token1');
			assert.equal(presets.length, 5);
			assert.equal(presets[0], '0');
			assert.equal(presets[1], '10');
			assert.equal(presets[2], '100');
			assert.equal(presets[3], '1000');
			assert.equal(presets[4], '10000');
		});

		it('should throw error if deposit token is not found in gui config', () => {
			expect(() => gui.getDepositPresets('token2')).toThrow(
				'Deposit token not found in gui config: token2'
			);
		});
	});

	describe('field value tests', async () => {
		let gui: DotrainOrderGui;
		beforeAll(async () => {
			mockServer
				.forPost('/rpc-url')
				.withBodyIncluding('0x82ad56cb')
				.thenSendJsonRpcResult(
					'0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000007546f6b656e203100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000025431000000000000000000000000000000000000000000000000000000000000'
				);
			gui = await DotrainOrderGui.chooseDeployment(dotrainWithGui, 'some-deployment');
		});

		it('should save the field value as presets', async () => {
			const allFieldDefinitions = gui.getAllFieldDefinitions();
			gui.saveFieldValue('binding-1', {
				isPreset: true,
				value: allFieldDefinitions[0].presets[0].id
			});
			assert.deepEqual(gui.getFieldValue('binding-1'), allFieldDefinitions[0].presets[0]);
			gui.saveFieldValue('binding-1', {
				isPreset: true,
				value: allFieldDefinitions[0].presets[1].id
			});
			assert.deepEqual(gui.getFieldValue('binding-1'), allFieldDefinitions[0].presets[1]);
			gui.saveFieldValue('binding-1', {
				isPreset: true,
				value: allFieldDefinitions[0].presets[2].id
			});
			assert.deepEqual(gui.getFieldValue('binding-1'), allFieldDefinitions[0].presets[2]);
		});

		it('should save field value as custom values', async () => {
			gui.saveFieldValues([
				{
					binding: 'binding-1',
					value: {
						isPreset: false,
						value: '0x1234567890abcdef1234567890abcdef12345678'
					}
				},
				{
					binding: 'binding-2',
					value: {
						isPreset: false,
						value: '100'
					}
				}
			]);
			gui.saveFieldValues([
				{
					binding: 'binding-1',
					value: {
						isPreset: false,
						value: 'some-string'
					}
				},
				{
					binding: 'binding-2',
					value: {
						isPreset: false,
						value: 'true'
					}
				}
			]);
			const fieldValues: AllFieldValuesResult[] = gui.getAllFieldValues();
			assert.equal(fieldValues.length, 2);
			assert.deepEqual(fieldValues[0], {
				binding: 'binding-1',
				value: {
					id: '',
					name: undefined,
					value: 'some-string'
				}
			});
			assert.deepEqual(fieldValues[1], {
				binding: 'binding-2',
				value: {
					id: '',
					name: undefined,
					value: 'true'
				}
			});
		});

		it('should throw error during save if preset is not found in field definition', () => {
			expect(() =>
				gui.saveFieldValue('binding-1', {
					isPreset: true,
					value: '89a3df5a-eee9-4af3-a10b-569f618f0f0c'
				})
			).toThrow('Invalid preset');
		});

		it('should throw error during save if field binding is not found in field definitions', () => {
			expect(() => gui.saveFieldValue('binding-3', { isPreset: false, value: '1' })).toThrow(
				'Field binding not found: binding-3'
			);
		});

		it('should get field value', async () => {
			gui.saveFieldValue('binding-1', {
				isPreset: false,
				value: '0x1234567890abcdef1234567890abcdef12345678'
			});
			let fieldValue = gui.getFieldValue('binding-1');
			assert.deepEqual(fieldValue, {
				id: '',
				name: undefined,
				value: '0x1234567890abcdef1234567890abcdef12345678'
			});

			gui.saveFieldValue('binding-2', { isPreset: false, value: 'true' });
			fieldValue = gui.getFieldValue('binding-2');
			assert.deepEqual(fieldValue, {
				id: '',
				name: undefined,
				value: 'true'
			});

			gui.saveFieldValue('binding-1', {
				isPreset: false,
				value: 'some-string'
			});
			fieldValue = gui.getFieldValue('binding-1');
			assert.deepEqual(fieldValue, {
				id: '',
				name: undefined,
				value: 'some-string'
			});

			gui.saveFieldValue('binding-2', { isPreset: false, value: '100.5' });
			fieldValue = gui.getFieldValue('binding-2');
			assert.deepEqual(fieldValue, {
				id: '',
				name: undefined,
				value: '100.5'
			});
		});

		it('should throw error during get if field binding is not found', () => {
			expect(() => gui.getFieldValue('binding-3')).toThrow('Field binding not found: binding-3');
		});
	});

	describe('field definition tests', async () => {
		let gui: DotrainOrderGui;
		beforeAll(async () => {
			mockServer
				.forPost('/rpc-url')
				.withBodyIncluding('0x82ad56cb')
				.thenSendJsonRpcResult(
					'0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000007546f6b656e203100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000025431000000000000000000000000000000000000000000000000000000000000'
				);
			gui = await DotrainOrderGui.chooseDeployment(dotrainWithGui, 'some-deployment');
		});

		it('should get field definition', async () => {
			const allFieldDefinitions = gui.getAllFieldDefinitions();
			assert.equal(allFieldDefinitions.length, 2);

			const fieldDefinition = gui.getFieldDefinition('binding-1');
			assert.equal(fieldDefinition.name, 'Field 1 name');
			assert.equal(fieldDefinition.description, 'Field 1 description');
			assert.equal(fieldDefinition.presets.length, 3);

			const preset1 = fieldDefinition.presets[0];
			assert.equal(preset1.name, 'Preset 1');
			assert.equal(preset1.value, '0x1234567890abcdef1234567890abcdef12345678');
			const preset2 = fieldDefinition.presets[1];
			assert.equal(preset2.name, 'Preset 2');
			assert.equal(preset2.value, 'false');
			const preset3 = fieldDefinition.presets[2];
			assert.equal(preset3.name, 'Preset 3');
			assert.equal(preset3.value, 'some-string');

			const fieldDefinition2 = gui.getFieldDefinition('binding-2');
			assert.equal(fieldDefinition2.presets[0].value, '99.2');
			assert.equal(fieldDefinition2.presets[1].value, '582.1');
			assert.equal(fieldDefinition2.presets[2].value, '648.239');
		});

		it('should throw error during get if field binding is not found', () => {
			expect(() => gui.getFieldDefinition('binding-3')).toThrow(
				'Field binding not found: binding-3'
			);
		});
	});

	describe('state management tests', async () => {
		let serializedState =
			'H4sIAAAAAAAA_3WNSQoCUQxEu1VEb-FaUPLHJDuP4BX-kC-N0IL2wuMrmHYhWJuXoag6dR9x4ALZsmuEJgZbqFJsXohLEQOVs22QRJBd8uJtxrdJCiasGMjRQnO2yjyMdRgvB9PrAfqNTue7PGTamf38eRrrfIhIDCmXKu3f_htuu1lLpQGYC9fK6XaV0XydK2WAY3wBw3-Y7v0AAAA=';
		let gui: DotrainOrderGui;
		beforeAll(async () => {
			mockServer
				.forPost('/rpc-url')
				.withBodyIncluding('0x82ad56cb')
				.thenSendJsonRpcResult(
					'0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000007546f6b656e203100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000025431000000000000000000000000000000000000000000000000000000000000'
				);
			gui = await DotrainOrderGui.chooseDeployment(dotrainWithGui, 'some-deployment');

			gui.saveFieldValue('binding-1', {
				isPreset: true,
				value: gui.getFieldDefinition('binding-1').presets[0].id
			});
			gui.saveFieldValue('binding-2', { isPreset: false, value: '100' });
			gui.saveDeposit('token1', '50.6');
		});

		it('should serialize gui state', async () => {
			const serialized = gui.serializeState();
			assert.equal(serialized, serializedState);
		});

		it('should deserialize gui state', async () => {
			gui.clearState();
			gui.deserializeState(serializedState);
			const fieldValues: AllFieldValuesResult[] = gui.getAllFieldValues();
			assert.equal(fieldValues.length, 2);
			assert.deepEqual(fieldValues[0], {
				binding: 'binding-1',
				value: {
					id: '0',
					name: 'Preset 1',
					value: '0x1234567890abcdef1234567890abcdef12345678'
				}
			});
			assert.deepEqual(fieldValues[1], {
				binding: 'binding-2',
				value: {
					id: '',
					name: undefined,
					value: '100'
				}
			});
			const deposits: TokenDeposit[] = gui.getDeposits();
			assert.equal(deposits.length, 1);
			assert.equal(deposits[0].token, 'token1');
			assert.equal(deposits[0].amount, '50.6');
			assert.equal(deposits[0].address, '0xc2132d05d31c914a87c6611c10748aeb04b58e8f');
		});

		it('should throw error during deserialize if config is different', async () => {
			// token1 info
			mockServer
				.forPost('/rpc-url')
				.once()
				.withBodyIncluding('0x82ad56cb')
				.thenSendJsonRpcResult(
					'0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000007546f6b656e203100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000025431000000000000000000000000000000000000000000000000000000000000'
				);
			// token2 info
			mockServer
				.forPost('/rpc-url')
				.once()
				.withBodyIncluding('0x82ad56cb')
				.thenSendJsonRpcResult(
					'0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000001a000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000120000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000754656b656e203200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000025432000000000000000000000000000000000000000000000000000000000000'
				);

			let dotrain2 = `
${guiConfig2}

${dotrain}
`;
			let gui2 = await DotrainOrderGui.chooseDeployment(dotrain2, 'other-deployment');
			let serialized = gui2.serializeState();
			expect(() => gui.deserializeState(serialized)).toThrow('Deserialized config mismatch');
		});

		it('should clear state', async () => {
			gui.clearState();
			const fieldValues: AllFieldValuesResult[] = gui.getAllFieldValues();
			assert.equal(fieldValues.length, 0);
			const deposits: TokenDeposit[] = gui.getDeposits();
			assert.equal(deposits.length, 0);
		});

		it('should check if field is preset', async () => {
			gui.saveFieldValue('binding-1', {
				isPreset: true,
				value: gui.getFieldDefinition('binding-1').presets[0].id
			});
			assert.equal(gui.isFieldPreset('binding-1'), true);
			gui.saveFieldValue('binding-2', {
				isPreset: false,
				value: '100'
			});
			assert.equal(gui.isFieldPreset('binding-2'), false);
		});

		it('should check if deposit is preset', async () => {
			gui.saveDeposit('token1', '55');
			assert.equal(gui.isDepositPreset('token1'), false);
			gui.saveDeposit('token1', '100');
			assert.equal(gui.isDepositPreset('token1'), true);
		});
	});

	describe('order operations tests', async () => {
		let gui: DotrainOrderGui;

		beforeEach(async () => {
			// token1 info
			mockServer
				.forPost('/rpc-url')
				.once()
				.withBodyIncluding('0x82ad56cb')
				.thenSendJsonRpcResult(
					'0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000007546f6b656e203100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000025431000000000000000000000000000000000000000000000000000000000000'
				);
			// token2 info
			mockServer
				.forPost('/rpc-url')
				.once()
				.withBodyIncluding('0x82ad56cb')
				.thenSendJsonRpcResult(
					'0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000001a000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000120000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000754656b656e203200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000025432000000000000000000000000000000000000000000000000000000000000'
				);

			let dotrain2 = `
      ${guiConfig2}
      
      ${dotrain}
      `;
			gui = await DotrainOrderGui.chooseDeployment(dotrain2, 'other-deployment');
		});

		it('checks input and output allowances', async () => {
			// token2 allowance
			await mockServer
				.forPost('/rpc-url')
				.withBodyIncluding('0x8f3cf7ad23cd3cadbd9735aff958023239c6a063')
				.thenSendJsonRpcResult(
					'0x0000000000000000000000000000000000000000000000000000000000000001'
				);

			gui.saveDeposit('token2', '200');

			const allowances: AllowancesResult = await gui.checkAllowances(
				'0x1234567890abcdef1234567890abcdef12345678'
			);
			assert.equal(allowances.length, 1);
			assert.equal(allowances[0].token, '0x8f3cf7ad23cd3cadbd9735aff958023239c6a063');
			assert.equal(allowances[0].allowance, '0x1');
		});

		it('generates approval calldatas', async () => {
			// token2 allowance - 1000 * 10^18
			await mockServer
				.forPost('/rpc-url')
				.thenSendJsonRpcResult(
					'0x00000000000000000000000000000000000000000000003635C9ADC5DEA00000'
				);

			gui.saveDeposit('token2', '5000');

			const approvalCalldatas: ApprovalCalldataResult = await gui.generateApprovalCalldatas(
				'0x1234567890abcdef1234567890abcdef12345678'
			);
			assert.equal(approvalCalldatas.length, 1);
			assert.equal(approvalCalldatas[0].token, '0x8f3cf7ad23cd3cadbd9735aff958023239c6a063');
			// 5000 - 1000 = 4000 * 10^18
			assert.equal(
				approvalCalldatas[0].calldata,
				'0x095ea7b3000000000000000000000000c95a5f8efe14d7a20bd2e5bafec4e71f8ce0b9a60000000000000000000000000000000000000000000000d8d726b7177a800000'
			);
		});

		it('generates deposit calldatas', async () => {
			await mockServer
				.forPost('/rpc-url')
				.withBodyIncluding('0xf0cfdd37')
				.thenSendJsonRpcResult(`0x${'0'.repeat(24) + '1'.repeat(40)}`);
			// iStore() call
			await mockServer
				.forPost('/rpc-url')
				.withBodyIncluding('0xc19423bc')
				.thenSendJsonRpcResult(`0x${'0'.repeat(24) + '2'.repeat(40)}`);
			// iParser() call
			await mockServer
				.forPost('/rpc-url')
				.withBodyIncluding('0x24376855')
				.thenSendJsonRpcResult(`0x${'0'.repeat(24) + '3'.repeat(40)}`);
			// parse2() call
			await mockServer
				.forPost('/rpc-url')
				.withBodyIncluding('0xa3869e14')
				// 0x1234 encoded bytes
				.thenSendJsonRpcResult(
					'0x000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000021234000000000000000000000000000000000000000000000000000000000000'
				);

			gui.saveDeposit('token2', '5000');

			const depositCalldatas: DepositCalldataResult = await gui.generateDepositCalldatas();
			assert.equal(depositCalldatas.length, 1);
			assert.equal(
				depositCalldatas[0],
				'0x91337c0a0000000000000000000000008f3cf7ad23cd3cadbd9735aff958023239c6a063000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000010f0cf064dd5920000000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000000'
			);
		});

		it('generates add order calldata', async () => {
			await mockServer
				.forPost('/rpc-url')
				.withBodyIncluding('0xf0cfdd37')
				.thenSendJsonRpcResult(`0x${'0'.repeat(24) + '1'.repeat(40)}`);
			// iStore() call
			await mockServer
				.forPost('/rpc-url')
				.withBodyIncluding('0xc19423bc')
				.thenSendJsonRpcResult(`0x${'0'.repeat(24) + '2'.repeat(40)}`);
			// iParser() call
			await mockServer
				.forPost('/rpc-url')
				.withBodyIncluding('0x24376855')
				.thenSendJsonRpcResult(`0x${'0'.repeat(24) + '3'.repeat(40)}`);
			// parse2() call
			await mockServer
				.forPost('/rpc-url')
				.withBodyIncluding('0xa3869e14')
				// 0x1234 encoded bytes
				.thenSendJsonRpcResult(
					'0x000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000021234000000000000000000000000000000000000000000000000000000000000'
				);

			gui.saveFieldValue('test-binding', {
				isPreset: false,
				value: '10'
			});

			const addOrderCalldata: AddOrderCalldataResult = await gui.generateAddOrderCalldata();
			assert.equal(addOrderCalldata.length, 2314);

			const currentDeployment: GuiDeployment = gui.getCurrentDeployment();
			assert.deepEqual(
				currentDeployment.deployment.scenario.bindings,
				new Map([['test-binding', '10']])
			);
		});

		it('should generate multicalldata for deposit and add order', async () => {
			await mockServer
				.forPost('/rpc-url')
				.withBodyIncluding('0xf0cfdd37')
				.thenSendJsonRpcResult(`0x${'0'.repeat(24) + '1'.repeat(40)}`);
			// iStore() call
			await mockServer
				.forPost('/rpc-url')
				.withBodyIncluding('0xc19423bc')
				.thenSendJsonRpcResult(`0x${'0'.repeat(24) + '2'.repeat(40)}`);
			// iParser() call
			await mockServer
				.forPost('/rpc-url')
				.withBodyIncluding('0x24376855')
				.thenSendJsonRpcResult(`0x${'0'.repeat(24) + '3'.repeat(40)}`);
			// parse2() call
			await mockServer
				.forPost('/rpc-url')
				.withBodyIncluding('0xa3869e14')
				// 0x1234 encoded bytes
				.thenSendJsonRpcResult(
					'0x000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000021234000000000000000000000000000000000000000000000000000000000000'
				);

			gui.saveDeposit('token2', '5000');

			gui.saveFieldValue('test-binding', {
				isPreset: true,
				value: '0'
			});

			const calldata: DepositAndAddOrderCalldataResult =
				await gui.generateDepositAndAddOrderCalldatas();
			assert.equal(calldata.length, 3146);

			const currentDeployment: GuiDeployment = gui.getCurrentDeployment();
			assert.deepEqual(
				currentDeployment.deployment.scenario.bindings,
				new Map([['test-binding', '0xbeef']])
			);
		});

		it('should throw error on order operations without selecting required tokens', async () => {
			let testDotrain = `
      ${guiConfig3}

      ${dotrain}
      `;
			let testGui = await DotrainOrderGui.chooseDeployment(testDotrain, 'other-deployment');

			await expect(async () =>
				testGui.checkAllowances('0x1234567890abcdef1234567890abcdef12345678')
			).rejects.toThrow('Token must be selected: token1');
			await expect(
				async () =>
					await testGui.generateApprovalCalldatas('0x1234567890abcdef1234567890abcdef12345678')
			).rejects.toThrow('Token must be selected: token1');
			await expect(async () => await testGui.generateDepositCalldatas()).rejects.toThrow(
				'Token must be selected: token1'
			);
			await expect(async () => await testGui.generateAddOrderCalldata()).rejects.toThrow(
				'Token must be selected: token1'
			);
			await expect(async () => await testGui.generateDepositAndAddOrderCalldatas()).rejects.toThrow(
				'Token must be selected: token1'
			);
		});

		it('should set vault ids', async () => {
			mockServer
				.forPost('/rpc-url')
				.withBodyIncluding('0x82ad56cb')
				.thenSendJsonRpcResult(
					'0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000007546f6b656e203100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000025431000000000000000000000000000000000000000000000000000000000000'
				);

			let testDotrain = `
          ${guiConfig2}
          
          ${dotrainWithoutVaultIds}
          `;
			gui = await DotrainOrderGui.chooseDeployment(testDotrain, 'other-deployment');

			let currentDeployment: GuiDeployment = gui.getCurrentDeployment();
			assert.equal(currentDeployment.deployment.order.inputs[0].vaultId, undefined);
			assert.equal(currentDeployment.deployment.order.outputs[0].vaultId, undefined);

			gui.setVaultId(true, 0, '0x123123123456456456');
			gui.setVaultId(false, 0, '0x123123123456456456');

			let newCurrentDeployment: GuiDeployment = gui.getCurrentDeployment();
			assert.notEqual(newCurrentDeployment.deployment.order.inputs[0].vaultId, undefined);
			assert.notEqual(newCurrentDeployment.deployment.order.outputs[0].vaultId, undefined);
			assert.equal(newCurrentDeployment.deployment.order.inputs[0].vaultId, '0x123123123456456456');
			assert.equal(
				newCurrentDeployment.deployment.order.outputs[0].vaultId,
				'0x123123123456456456'
			);
		});

		it('should skip deposits with zero amount for deposit calldata', async () => {
			// token1 info
			mockServer
				.forPost('/rpc-url')
				.once()
				.withBodyIncluding('0x82ad56cb')
				.thenSendJsonRpcResult(
					'0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000007546f6b656e203100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000025431000000000000000000000000000000000000000000000000000000000000'
				);
			// token2 info
			mockServer
				.forPost('/rpc-url')
				.once()
				.withBodyIncluding('0x82ad56cb')
				.thenSendJsonRpcResult(
					'0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000001a000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000120000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000754656b656e203200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000025432000000000000000000000000000000000000000000000000000000000000'
				);

			gui.saveDeposit('token1', '0');
			gui.saveDeposit('token2', '0');
			const calldatas = await gui.generateDepositCalldatas();
			assert.equal(calldatas.length, 0);
		});
	});

	describe('select tokens tests', async () => {
		let gui: DotrainOrderGui;
		beforeAll(async () => {
			let dotrain3 = `
      ${guiConfig3}

      ${dotrain}
      `;
			gui = await DotrainOrderGui.chooseDeployment(dotrain3, 'other-deployment');
		});

		it('should get select tokens', async () => {
			const selectTokens: SelectTokens = gui.getSelectTokens();
			assert.equal(selectTokens.size, 2);
			assert.equal(selectTokens.get('token1'), '0x0000000000000000000000000000000000000000');
			assert.equal(selectTokens.get('token2'), '0x0000000000000000000000000000000000000000');
		});

		it('should throw error if select tokens not set', async () => {
			mockServer
				.forPost('/rpc-url')
				.once()
				.withBodyIncluding('0x82ad56cb')
				.thenSendJsonRpcResult(
					'0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000007546f6b656e203100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000025431000000000000000000000000000000000000000000000000000000000000'
				);
			let testGui = await DotrainOrderGui.chooseDeployment(dotrainWithGui, 'some-deployment');

			expect(() => testGui.getSelectTokens()).toThrow('Select tokens not set');
			await expect(
				async () => await testGui.saveSelectTokenAddress('token1', '0x1')
			).rejects.toThrow('Select tokens not set');
		});

		it('should throw error if token not found', async () => {
			await expect(async () => await gui.saveSelectTokenAddress('token3', '0x1')).rejects.toThrow(
				'Token not found'
			);
		});

		it('should save select token address', async () => {
			mockServer
				.forPost('/rpc-url')
				.once()
				.withBodyIncluding('0x82ad56cb')
				.thenSendJsonRpcResult(
					'0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000007546f6b656e203100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000025431000000000000000000000000000000000000000000000000000000000000'
				);
			// token2 info
			mockServer
				.forPost('/rpc-url')
				.once()
				.withBodyIncluding('0x82ad56cb')
				.thenSendJsonRpcResult(
					'0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000001a000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000120000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000754656b656e203200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000025432000000000000000000000000000000000000000000000000000000000000'
				);

			let initialTokenInfo: TokenInfos = await gui.getTokenInfos();
			assert.equal(initialTokenInfo.size, 0);

			let dotrainConfig: Config = gui.getDotrainConfig();
			assert.equal(
				dotrainConfig.tokens.get('token1')?.address,
				'0xc2132d05d31c914a87c6611c10748aeb04b58e8f'
			);
			assert.equal(
				dotrainConfig.tokens.get('token2')?.address,
				'0x8f3cf7ad23cd3cadbd9735aff958023239c6a063'
			);

			let currentDeployment: GuiDeployment = gui.getCurrentDeployment();
			assert.equal(
				currentDeployment.deployment.order.inputs[0].token.address,
				'0xc2132d05d31c914a87c6611c10748aeb04b58e8f'
			);
			assert.equal(
				currentDeployment.deployment.order.outputs[0].token.address,
				'0x8f3cf7ad23cd3cadbd9735aff958023239c6a063'
			);

			await gui.saveSelectTokenAddress('token1', '0x6666666666666666666666666666666666666666');
			await gui.saveSelectTokenAddress('token2', '0x8888888888888888888888888888888888888888');
			assert.equal(
				gui.getSelectTokens().get('token1'),
				'0x6666666666666666666666666666666666666666'
			);
			assert.equal(
				gui.getSelectTokens().get('token2'),
				'0x8888888888888888888888888888888888888888'
			);

			let tokenInfo: TokenInfos = await gui.getTokenInfos();
			assert.equal(tokenInfo.size, 2);

			let newDotrainConfig: Config = gui.getDotrainConfig();
			assert.equal(
				newDotrainConfig.tokens.get('token1')?.address,
				'0x6666666666666666666666666666666666666666'
			);
			assert.equal(
				newDotrainConfig.tokens.get('token2')?.address,
				'0x8888888888888888888888888888888888888888'
			);

			let newCurrentDeployment: GuiDeployment = gui.getCurrentDeployment();
			assert.equal(
				newCurrentDeployment.deployment.order.inputs[0].token.address,
				'0x6666666666666666666666666666666666666666'
			);
			assert.equal(
				newCurrentDeployment.deployment.order.outputs[0].token.address,
				'0x8888888888888888888888888888888888888888'
			);
		});
	});
});
