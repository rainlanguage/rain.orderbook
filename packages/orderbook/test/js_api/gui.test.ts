import assert from 'assert';
import { afterAll, beforeAll, beforeEach, describe, expect, it, Mock, vi } from 'vitest';
import { DotrainOrderGui } from '../../dist/cjs/js_api.js';
import {
	AddOrderCalldataResult,
	AllFieldValuesResult,
	AllowancesResult,
	AllTokenInfos,
	DeploymentDetails,
	DeploymentKeys,
	DeploymentTransactionArgs,
	DepositAndAddOrderCalldataResult,
	GuiCfg,
	GuiDeploymentCfg,
	IOVaultIds,
	NameAndDescriptionCfg,
	TokenDeposit,
	TokenInfo
} from '../../dist/types/js_api.js';
import { getLocal } from 'mockttp';

const guiConfig = `
gui:
  name: Fixed limit
  description: Fixed limit order strategy
  short-description: Buy WETH with USDC on Base.
  deployments:
    some-deployment:
      name: Buy WETH with USDC on Base.
      description: Buy WETH with USDC for fixed price on Base network.
      short-description: Buy WETH with USDC on Base.
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
    other-deployment:
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
    other-deployment:
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
    other-deployment:
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
        - key: token1
        - key: token2
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
metaboards:
    test: https://metaboard.com

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
        deployer: some-deployer
        bindings:
            test-binding: 5
        scenarios:
            sub-scenario:
                bindings:
                    another-binding: 300

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
        scenario: some-scenario.sub-scenario
        order: some-order
---
#test-binding !
#another-binding !
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
metaboards:
    test: https://metaboard.com

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
        deployer: some-deployer
        bindings:
            test-binding: 5

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
#test-binding !
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
:;
`;
const dotrainWithoutTokens = `
networks:
    some-network:
        rpc: http://localhost:8085/rpc-url
        chain-id: 123
        network-id: 123
        currency: ETH

subgraphs:
    some-sg: https://www.some-sg.com
metaboards:
    test: https://metaboard.com

deployers:
    some-deployer:
        network: some-network
        address: 0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba

orderbooks:
    some-orderbook:
        address: 0xc95A5f8eFe14d7a20BD2E5BAFEC4E71f8Ce0B9A6
        network: some-network
        subgraph: some-sg

scenarios:
    some-scenario:
        deployer: some-deployer
        bindings:
            test-binding: 5

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
		const deployments: DeploymentKeys = await DotrainOrderGui.getDeploymentKeys(dotrainWithGui);
		assert.equal(deployments.length, 2);
		assert.equal(deployments[0], 'some-deployment');
		assert.equal(deployments[1], 'other-deployment');
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

		const guiConfig = gui.getGuiConfig() as GuiCfg;
		assert.equal(guiConfig.name, 'Fixed limit');
		assert.equal(guiConfig.description, 'Fixed limit order strategy');
	});

	it('should initialize gui object with state update callback', async () => {
		const stateUpdateCallback = vi.fn();
		const gui = await DotrainOrderGui.chooseDeployment(
			dotrainWithGui,
			'some-deployment',
			stateUpdateCallback
		);

		gui.executeStateUpdateCallback();
		assert.equal(stateUpdateCallback.mock.calls.length, 1);
	});

	it('should get strategy details', async () => {
		const strategyDetails: NameAndDescriptionCfg =
			await DotrainOrderGui.getStrategyDetails(dotrainWithGui);
		assert.equal(strategyDetails.name, 'Fixed limit');
		assert.equal(strategyDetails.description, 'Fixed limit order strategy');
		assert.equal(strategyDetails.short_description, 'Buy WETH with USDC on Base.');
	});

	it('should get deployment details', async () => {
		const deploymentDetails: DeploymentDetails =
			await DotrainOrderGui.getDeploymentDetails(dotrainWithGui);
		const entries = Array.from(deploymentDetails.entries());
		assert.equal(entries[0][0], 'other-deployment');
		assert.equal(entries[0][1].name, 'Test test');
		assert.equal(entries[0][1].description, 'Test test test');
		assert.equal(entries[1][0], 'some-deployment');
		assert.equal(entries[1][1].name, 'Buy WETH with USDC on Base.');
		assert.equal(entries[1][1].description, 'Buy WETH with USDC for fixed price on Base network.');
	});

	it('should get deployment detail', async () => {
		const deploymentDetail: NameAndDescriptionCfg = await DotrainOrderGui.getDeploymentDetail(
			dotrainWithGui,
			'other-deployment'
		);
		assert.equal(deploymentDetail.name, 'Test test');
		assert.equal(deploymentDetail.description, 'Test test test');
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

		let token1TokenInfo = await gui.getTokenInfo('token1');
		let token2TokenInfo = await gui.getTokenInfo('token2');
		assert.equal(token1TokenInfo.address, '0xc2132d05d31c914a87c6611c10748aeb04b58e8f');
		assert.equal(token1TokenInfo.decimals, 6);
		assert.equal(token1TokenInfo.name, 'Token 1');
		assert.equal(token1TokenInfo.symbol, 'T1');
		assert.equal(token2TokenInfo.address, '0x8f3cf7ad23cd3cadbd9735aff958023239c6a063');
		assert.equal(token2TokenInfo.decimals, 18);
		assert.equal(token2TokenInfo.name, 'Token 2');
		assert.equal(token2TokenInfo.symbol, 'T2');
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

		let allTokenInfos: AllTokenInfos = await gui.getAllTokenInfos();
		assert.equal(allTokenInfos.length, 2);
		assert.equal(allTokenInfos[0].address, '0xc2132d05d31c914a87c6611c10748aeb04b58e8f');
		assert.equal(allTokenInfos[0].decimals, 6);
		assert.equal(allTokenInfos[0].name, 'Token 1');
		assert.equal(allTokenInfos[0].symbol, 'T1');
		assert.equal(allTokenInfos[1].address, '0x8f3cf7ad23cd3cadbd9735aff958023239c6a063');
		assert.equal(allTokenInfos[1].decimals, 18);
		assert.equal(allTokenInfos[1].name, 'Token 2');
		assert.equal(allTokenInfos[1].symbol, 'T2');
	});

	describe('deposit tests', async () => {
		let gui: DotrainOrderGui;
		let stateUpdateCallback: Mock;
		beforeEach(async () => {
			stateUpdateCallback = vi.fn();
			mockServer
				.forPost('/rpc-url')
				.withBodyIncluding('0x82ad56cb')
				.thenSendJsonRpcResult(
					'0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000007546f6b656e203100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000025431000000000000000000000000000000000000000000000000000000000000'
				);
			gui = await DotrainOrderGui.chooseDeployment(
				dotrainWithGui,
				'some-deployment',
				stateUpdateCallback
			);
		});

		it('should add deposit', async () => {
			assert.equal(gui.hasAnyDeposit(), false);

			gui.saveDeposit('token1', '50.6');
			const deposits: TokenDeposit[] = gui.getDeposits();
			assert.equal(deposits.length, 1);

			assert.equal(gui.hasAnyDeposit(), true);

			assert.equal(stateUpdateCallback.mock.calls.length, 1);
			expect(stateUpdateCallback).toHaveBeenCalledWith(gui.serializeState());
		});

		it('should update deposit', async () => {
			gui.saveDeposit('token1', '50.6');
			gui.saveDeposit('token1', '100.6');
			const deposits: TokenDeposit[] = gui.getDeposits();
			assert.equal(deposits.length, 1);
			assert.equal(deposits[0].amount, '100.6');

			assert.equal(stateUpdateCallback.mock.calls.length, 2);
			expect(stateUpdateCallback).toHaveBeenCalledWith(gui.serializeState());
		});

		it('should throw error if deposit token is not found in gui config', () => {
			expect(() => gui.saveDeposit('token3', '1')).toThrow(
				'Deposit token not found in gui config: token3'
			);
		});

		it('should remove deposit', async () => {
			gui.saveDeposit('token1', '50.6');
			let deposits: TokenDeposit[] = gui.getDeposits();
			assert.equal(deposits.length, 1);

			gui.removeDeposit('token1');
			let depositsAfterRemove: TokenDeposit[] = gui.getDeposits();
			assert.equal(depositsAfterRemove.length, 0);

			gui.saveDeposit('token1', '50.6');
			assert.equal(gui.getDeposits().length, 1);
			gui.saveDeposit('token1', '');
			assert.equal(gui.getDeposits().length, 0);

			assert.equal(stateUpdateCallback.mock.calls.length, 4);
			expect(stateUpdateCallback).toHaveBeenCalledWith(gui.serializeState());
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
		let stateUpdateCallback: Mock;
		beforeEach(async () => {
			stateUpdateCallback = vi.fn();
			mockServer
				.forPost('/rpc-url')
				.withBodyIncluding('0x82ad56cb')
				.thenSendJsonRpcResult(
					'0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000007546f6b656e203100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000025431000000000000000000000000000000000000000000000000000000000000'
				);
			gui = await DotrainOrderGui.chooseDeployment(
				dotrainWithGui,
				'some-deployment',
				stateUpdateCallback
			);
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

			assert.equal(stateUpdateCallback.mock.calls.length, 3);
			expect(stateUpdateCallback).toHaveBeenCalledWith(gui.serializeState());
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

			assert.equal(stateUpdateCallback.mock.calls.length, 4);
			expect(stateUpdateCallback).toHaveBeenCalledWith(gui.serializeState());
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
			'H4sIAAAAAAAA_7WOu07DMBSG44KKhBgQYkVCYiXEcUgwVdm4pEFCRcpQsaWpIVFcO7WdIuAhGFl5gYonYGXjeRAbQti0qbr2LOc_5z-XD1h_saGzIlLZ_ZwNcnYHdA9a67PuOKEVaehO0zi8IMy1TKzq7MODoDaC_kdWdHYhXHysXhlAyYfEZkTdc1Hs6F6mVNlyHMrThGZcqhaG2HdEmdqVoE_mITAKmNdncbit5e2CAE2wpu34l2HXBYY0rtOherVk1uf212Tvsz15f_HfvnsNdPzxmoKtOVY0ZUVGzZzzPA9MqyAI9rUMZYSPbOzRccRE1VM33c7hIz-PqpG8CPvXpz68xKNOcpV3i5NNvcNVRoQ9ICXlD0PC1A_3nQ2PVQIAAA==';
		let dotrain3: string;
		let gui: DotrainOrderGui;
		beforeAll(async () => {
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
					'0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000007546f6b656e203100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000025431000000000000000000000000000000000000000000000000000000000000'
				);
			mockServer
				.forPost('/rpc-url')
				.once()
				.withBodyIncluding('0x82ad56cb')
				.thenSendJsonRpcResult(
					'0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000001a000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000754656b656e203200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000025432000000000000000000000000000000000000000000000000000000000000'
				);
			dotrain3 = `${guiConfig3}

${dotrain}`;
			gui = await DotrainOrderGui.chooseDeployment(dotrain3, 'other-deployment');

			gui.saveFieldValue('test-binding', {
				isPreset: true,
				value: gui.getFieldDefinition('test-binding').presets[0].id
			});
			gui.saveDeposit('token1', '50.6');
			gui.saveDeposit('token2', '100');
			gui.removeSelectToken('token1');
			await gui.saveSelectToken('token1', '0x6666666666666666666666666666666666666666');
			gui.setVaultId(true, 0, '666');
			gui.setVaultId(false, 0, '333');
		});

		it('should serialize gui state', async () => {
			const serialized = gui.serializeState();
			assert.equal(serialized, serializedState);
		});

		it('should deserialize gui state', async () => {
			let gui = await DotrainOrderGui.deserializeState(dotrain3, serializedState);

			const fieldValues: AllFieldValuesResult[] = gui.getAllFieldValues();
			assert.equal(fieldValues.length, 1);
			assert.deepEqual(fieldValues[0], {
				binding: 'test-binding',
				value: {
					id: '0',
					name: undefined,
					value: 'test-value'
				}
			});

			assert.equal(gui.isSelectTokenSet('token1'), true);
			assert.equal(gui.isSelectTokenSet('token2'), true);
			const deposits: TokenDeposit[] = gui.getDeposits();
			assert.equal(deposits.length, 2);
			assert.equal(deposits[0].token, 'token1');
			assert.equal(deposits[0].amount, '50.6');
			assert.equal(deposits[0].address, '0x6666666666666666666666666666666666666666');
			assert.equal(deposits[1].token, 'token2');
			assert.equal(deposits[1].amount, '100');
			assert.equal(deposits[1].address, '0x8f3cf7ad23cd3cadbd9735aff958023239c6a063');

			let guiDeployment: GuiDeploymentCfg = gui.getCurrentDeployment();
			assert.equal(guiDeployment.deployment.order.inputs[0].vaultId, '0x29a');
			assert.equal(guiDeployment.deployment.order.outputs[0].vaultId, '0x14d');
		});

		it('should throw error if given dotrain is different', async () => {
			let testDotrain = `${guiConfig}

${dotrainWithoutTokens}`;
			await expect(
				async () => await DotrainOrderGui.deserializeState(testDotrain, serializedState)
			).rejects.toThrow('Deserialized dotrain mismatch');
		});

		it('should clear state', async () => {
			gui.clearState();
			const fieldValues: AllFieldValuesResult[] = gui.getAllFieldValues();
			assert.equal(fieldValues.length, 0);
			const deposits: TokenDeposit[] = gui.getDeposits();
			assert.equal(deposits.length, 0);
		});

		it('should check if field is preset', async () => {
			gui.saveFieldValue('test-binding', {
				isPreset: true,
				value: gui.getFieldDefinition('test-binding').presets[0].id
			});
			assert.equal(gui.isFieldPreset('test-binding'), true);
			gui.saveFieldValue('test-binding', {
				isPreset: false,
				value: '100'
			});
			assert.equal(gui.isFieldPreset('test-binding'), false);
		});

		it('should check if deposit is preset', async () => {
			gui.saveDeposit('token1', '55');
			assert.equal(gui.isDepositPreset('token1'), false);
			gui.saveDeposit('token1', '0');
			assert.equal(gui.isDepositPreset('token1'), true);
		});

		it('should keep the same vault ids after deserializing if not set during serializing', async () => {
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
			let deployment: GuiDeploymentCfg = gui.getCurrentDeployment();
			assert.equal(deployment.deployment.order.inputs[0].vaultId, undefined);
			assert.equal(deployment.deployment.order.outputs[0].vaultId, undefined);

			gui = await DotrainOrderGui.deserializeState(testDotrain, gui.serializeState());

			deployment = gui.getCurrentDeployment();
			assert.equal(deployment.deployment.order.inputs[0].vaultId, undefined);
			assert.equal(deployment.deployment.order.outputs[0].vaultId, undefined);
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

			gui.saveDeposit('token1', '1000');
			gui.saveDeposit('token2', '5000');

			const result = await gui.generateApprovalCalldatas(
				'0x1234567890abcdef1234567890abcdef12345678'
			);

			assert.equal(result.Calldatas.length, 1);
			assert.equal(result.Calldatas[0].token, '0x8f3cf7ad23cd3cadbd9735aff958023239c6a063');
			assert.equal(
				result.Calldatas[0].calldata,
				'0x095ea7b3000000000000000000000000c95a5f8efe14d7a20bd2e5bafec4e71f8ce0b9a600000000000000000000000000000000000000000000010f0cf064dd59200000'
			);

			// Test no deposits case
			gui.removeDeposit('token1');
			gui.removeDeposit('token2');
			const emptyResult = await gui.generateApprovalCalldatas(
				'0x1234567890abcdef1234567890abcdef12345678'
			);
			assert.equal(emptyResult, 'NoDeposits');
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

			gui.saveDeposit('token1', '1000');
			gui.saveDeposit('token2', '5000');

			const result = await gui.generateDepositCalldatas();

			assert.equal(result.Calldatas.length, 1);
			assert.equal(
				result.Calldatas[0],
				'0x91337c0a0000000000000000000000008f3cf7ad23cd3cadbd9735aff958023239c6a063000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000010f0cf064dd5920000000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000000'
			);

			// Test no deposits case
			gui.removeDeposit('token1');
			gui.removeDeposit('token2');
			const emptyResult = await gui.generateDepositCalldatas();
			assert.equal(emptyResult, 'NoDeposits');
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

			const currentDeployment: GuiDeploymentCfg = gui.getCurrentDeployment();
			assert.deepEqual(currentDeployment.deployment.scenario.bindings, {
				'test-binding': '10',
				'another-binding': '300'
			});
		});

		it('should generate multicalldata for deposit and add order with existing vault ids', async () => {
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

			gui.saveDeposit('token1', '1000');
			gui.saveDeposit('token2', '5000');

			gui.saveFieldValue('test-binding', {
				isPreset: true,
				value: '0'
			});

			const calldata: DepositAndAddOrderCalldataResult =
				await gui.generateDepositAndAddOrderCalldatas();
			assert.equal(calldata.length, 3146);

			const currentDeployment: GuiDeploymentCfg = gui.getCurrentDeployment();
			assert.deepEqual(currentDeployment.deployment.scenario.bindings, {
				'test-binding': '0xbeef',
				'another-binding': '300'
			});
		});

		it('should generate multicalldata for deposit and add order with without vault ids', async () => {
			await mockServer
				.forPost('/rpc-url')
				.once()
				.withBodyIncluding('0x82ad56cb')
				.thenSendJsonRpcResult(
					'0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000007546f6b656e203100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000025431000000000000000000000000000000000000000000000000000000000000'
				);
			await mockServer
				.forPost('/rpc-url')
				.once()
				.withBodyIncluding('0x82ad56cb')
				.thenSendJsonRpcResult(
					'0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000001a000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000754656b656e203200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000025432000000000000000000000000000000000000000000000000000000000000'
				);

			let testDotrain = `${guiConfig2}

${dotrainWithoutVaultIds}`;
			let gui = await DotrainOrderGui.chooseDeployment(testDotrain, 'other-deployment');

			await mockServer
				.forPost('/rpc-url')
				.withBodyIncluding('0xf0cfdd37')
				.thenSendJsonRpcResult(`0x${'0'.repeat(24) + '1'.repeat(40)}`);
			await mockServer
				.forPost('/rpc-url')
				.withBodyIncluding('0xc19423bc')
				.thenSendJsonRpcResult(`0x${'0'.repeat(24) + '2'.repeat(40)}`);
			await mockServer
				.forPost('/rpc-url')
				.withBodyIncluding('0x24376855')
				.thenSendJsonRpcResult(`0x${'0'.repeat(24) + '3'.repeat(40)}`);
			await mockServer
				.forPost('/rpc-url')
				.withBodyIncluding('0xa3869e14')
				// 0x1234 encoded bytes
				.thenSendJsonRpcResult(
					'0x000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000021234000000000000000000000000000000000000000000000000000000000000'
				);

			gui.saveDeposit('token1', '1000');
			gui.saveDeposit('token2', '5000');

			gui.saveFieldValue('test-binding', {
				isPreset: true,
				value: '0'
			});

			const calldata: DepositAndAddOrderCalldataResult =
				await gui.generateDepositAndAddOrderCalldatas();
			assert.equal(calldata.length, 3146);

			const currentDeployment: GuiDeploymentCfg = gui.getCurrentDeployment();
			assert.equal(
				currentDeployment.deployment.order.inputs[0].vaultId,
				currentDeployment.deployment.order.outputs[0].vaultId
			);
		});

		it('should throw error on order operations without selecting required tokens', async () => {
			let testDotrain = `
      ${guiConfig3}

      ${dotrainWithoutTokens}
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

		it('should throw error if field value not set', async () => {
			await mockServer
				.forPost('/rpc-url')
				.once()
				.withBodyIncluding('0x82ad56cb')
				.thenSendJsonRpcResult(
					'0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000007546f6b656e203100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000025431000000000000000000000000000000000000000000000000000000000000'
				);
			await mockServer
				.forPost('/rpc-url')
				.once()
				.withBodyIncluding('0x82ad56cb')
				.thenSendJsonRpcResult(
					'0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000001a000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000754656b656e203200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000025432000000000000000000000000000000000000000000000000000000000000'
				);

			let testDotrain = `${guiConfig2}

${dotrainWithoutVaultIds}`;
			let gui = await DotrainOrderGui.chooseDeployment(testDotrain, 'other-deployment');

			gui.saveDeposit('token1', '1000');
			gui.saveDeposit('token2', '5000');

			await expect(async () => await gui.generateAddOrderCalldata()).rejects.toThrow(
				'Missing field value: Test binding'
			);
			await expect(async () => await gui.generateDepositAndAddOrderCalldatas()).rejects.toThrow(
				'Missing field value: Test binding'
			);

			let missingFieldValues = gui.getMissingFieldValues();
			assert.equal(missingFieldValues.length, 1);
			assert.equal(missingFieldValues[0], 'Test binding');
		});

		it('should set vault ids', async () => {
			let stateUpdateCallback = vi.fn();
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
			gui = await DotrainOrderGui.chooseDeployment(
				testDotrain,
				'other-deployment',
				stateUpdateCallback
			);

			let currentDeployment: GuiDeploymentCfg = gui.getCurrentDeployment();
			assert.equal(currentDeployment.deployment.order.inputs[0].vaultId, undefined);
			assert.equal(currentDeployment.deployment.order.outputs[0].vaultId, undefined);

			assert.equal(gui.hasAnyVaultId(), false);

			gui.setVaultId(true, 0, '0x123');

			assert.equal(gui.hasAnyVaultId(), true);

			assert.equal(gui.getVaultIds().get('input')?.[0], '0x123');
			assert.equal(gui.getVaultIds().get('output')?.[0], undefined);

			gui.setVaultId(false, 0, '0x234');

			let newCurrentDeployment: GuiDeploymentCfg = gui.getCurrentDeployment();
			assert.notEqual(newCurrentDeployment.deployment.order.inputs[0].vaultId, undefined);
			assert.notEqual(newCurrentDeployment.deployment.order.outputs[0].vaultId, undefined);
			assert.equal(newCurrentDeployment.deployment.order.inputs[0].vaultId, '0x123');
			assert.equal(newCurrentDeployment.deployment.order.outputs[0].vaultId, '0x234');

			const vaultIds: IOVaultIds = gui.getVaultIds();
			assert.equal(vaultIds.get('input')?.[0], '0x123');
			assert.equal(vaultIds.get('output')?.[0], '0x234');

			gui.setVaultId(true, 0, undefined);
			assert.equal(gui.getVaultIds().get('input')?.[0], undefined);

			gui.setVaultId(false, 0, '');
			assert.equal(gui.getVaultIds().get('output')?.[0], undefined);

			expect(() => gui.setVaultId(true, 0, 'test')).toThrow(
				"Invalid value for field 'vault-id': Failed to parse vault id in index '0' of inputs in order 'some-order'"
			);

			assert.equal(stateUpdateCallback.mock.calls.length, 4);
			expect(stateUpdateCallback).toHaveBeenCalledWith(gui.serializeState());
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
			assert.equal(calldatas.Calldatas.length, 0);
		});

		it('should generate deployment transaction args', async () => {
			await mockServer
				.forPost('/rpc-url')
				.thenSendJsonRpcResult(
					'0x00000000000000000000000000000000000000000000003635C9ADC5DEA00000'
				);
			await mockServer
				.forPost('/rpc-url')
				.withBodyIncluding('0xf0cfdd37')
				.thenSendJsonRpcResult(`0x${'0'.repeat(24) + '1'.repeat(40)}`);
			await mockServer
				.forPost('/rpc-url')
				.withBodyIncluding('0xc19423bc')
				.thenSendJsonRpcResult(`0x${'0'.repeat(24) + '2'.repeat(40)}`);
			await mockServer
				.forPost('/rpc-url')
				.withBodyIncluding('0x24376855')
				.thenSendJsonRpcResult(`0x${'0'.repeat(24) + '3'.repeat(40)}`);
			await mockServer
				.forPost('/rpc-url')
				.withBodyIncluding('0xa3869e14')
				.thenSendJsonRpcResult(
					'0x000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000021234000000000000000000000000000000000000000000000000000000000000'
				);

			gui.saveDeposit('token2', '5000');
			gui.saveFieldValue('test-binding', {
				isPreset: false,
				value: '10'
			});

			let result: DeploymentTransactionArgs = await gui.getDeploymentTransactionArgs(
				'0x1234567890abcdef1234567890abcdef12345678'
			);

			assert.equal(result.approvals.length, 1);
			assert.equal(
				result.approvals[0].calldata,
				'0x095ea7b3000000000000000000000000c95a5f8efe14d7a20bd2e5bafec4e71f8ce0b9a600000000000000000000000000000000000000000000010f0cf064dd59200000'
			);
			assert.equal(result.approvals[0].symbol, 'T2');
			assert.equal(result.deploymentCalldata.length, 3146);
			assert.equal(result.orderbookAddress, '0xc95a5f8efe14d7a20bd2e5bafec4e71f8ce0b9a6');
			assert.equal(result.chainId, 123);

			gui.removeDeposit('token2');
			result = await gui.getDeploymentTransactionArgs('0x1234567890abcdef1234567890abcdef12345678');

			assert.equal(result.approvals.length, 0);
			assert.equal(result.deploymentCalldata.length, 2634);
			assert.equal(result.orderbookAddress, '0xc95a5f8efe14d7a20bd2e5bafec4e71f8ce0b9a6');
			assert.equal(result.chainId, 123);
		});
	});

	describe('select tokens tests', async () => {
		let gui: DotrainOrderGui;
		let stateUpdateCallback: Mock;
		beforeEach(async () => {
			stateUpdateCallback = vi.fn();
			let dotrain3 = `
      ${guiConfig3}

      ${dotrainWithoutTokens}
      `;
			gui = await DotrainOrderGui.chooseDeployment(
				dotrain3,
				'other-deployment',
				stateUpdateCallback
			);
		});

		it('should get select tokens', async () => {
			const selectTokens = gui.getSelectTokens();
			assert.equal(selectTokens.length, 2);
			assert.equal(selectTokens[0].key, 'token1');
			assert.equal(selectTokens[1].key, 'token2');
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

			assert.equal(testGui.getSelectTokens().length, 0);
			await expect(async () => await testGui.saveSelectToken('token1', '0x1')).rejects.toThrow(
				'Select tokens not set'
			);
		});

		it('should throw error if token not found', async () => {
			await expect(async () => await gui.saveSelectToken('token3', '0x1')).rejects.toThrow(
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

			assert.equal(gui.isSelectTokenSet('token1'), false);
			assert.equal(gui.isSelectTokenSet('token2'), false);
			assert.equal(gui.areAllTokensSelected(), false);

			await expect(async () => await gui.getTokenInfo('token1')).rejects.toThrow(
				"Missing required field 'tokens' in root"
			);
			await expect(async () => await gui.getTokenInfo('token2')).rejects.toThrow(
				"Missing required field 'tokens' in root"
			);

			await gui.saveSelectToken('token1', '0x6666666666666666666666666666666666666666');
			await gui.saveSelectToken('token2', '0x8888888888888888888888888888888888888888');

			assert.equal(gui.isSelectTokenSet('token1'), true);
			assert.equal(gui.isSelectTokenSet('token2'), true);
			assert.equal(gui.areAllTokensSelected(), true);

			let tokenInfo: TokenInfo = await gui.getTokenInfo('token1');
			assert.equal(tokenInfo.name, 'Token 1');
			assert.equal(tokenInfo.symbol, 'T1');
			assert.equal(tokenInfo.decimals, 6);

			tokenInfo = await gui.getTokenInfo('token2');
			assert.equal(tokenInfo.name, 'Teken 2');
			assert.equal(tokenInfo.symbol, 'T2');
			assert.equal(tokenInfo.decimals, 18);

			assert.equal(stateUpdateCallback.mock.calls.length, 2);
			expect(stateUpdateCallback).toHaveBeenCalledWith(gui.serializeState());
		});

		it('should replace select token', async () => {
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
					'0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000001a000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000120000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000754656b656e203200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000025432000000000000000000000000000000000000000000000000000000000000'
				);

			await gui.saveSelectToken('token1', '0x6666666666666666666666666666666666666666');
			assert.equal(gui.isSelectTokenSet('token1'), true);
			let tokenInfo = await gui.getTokenInfo('token1');
			assert.equal(tokenInfo.name, 'Token 1');
			assert.equal(tokenInfo.symbol, 'T1');
			assert.equal(tokenInfo.decimals, 6);

			await gui.replaceSelectToken('token1', '0x8888888888888888888888888888888888888888');
			assert.equal(gui.isSelectTokenSet('token1'), true);
			tokenInfo = await gui.getTokenInfo('token1');
			assert.equal(tokenInfo.name, 'Teken 2');
			assert.equal(tokenInfo.symbol, 'T2');
			assert.equal(tokenInfo.decimals, 18);

			assert.equal(stateUpdateCallback.mock.calls.length, 3);
			expect(stateUpdateCallback).toHaveBeenCalledWith(gui.serializeState());
		});

		it('should remove select token', async () => {
			stateUpdateCallback = vi.fn();
			let dotrain3 = `
      ${guiConfig3}

      ${dotrainWithoutTokens}
      `;
			gui = await DotrainOrderGui.chooseDeployment(
				dotrain3,
				'other-deployment',
				stateUpdateCallback
			);

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
					'0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000001a000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000120000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000754656b656e203200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000025432000000000000000000000000000000000000000000000000000000000000'
				);

			await gui.saveSelectToken('token1', '0x6666666666666666666666666666666666666666');
			assert.equal(gui.isSelectTokenSet('token1'), true);
			let tokenInfo = await gui.getTokenInfo('token1');
			assert.equal(tokenInfo.name, 'Token 1');
			assert.equal(tokenInfo.symbol, 'T1');
			assert.equal(tokenInfo.decimals, 6);

			gui.removeSelectToken('token1');
			assert.equal(gui.isSelectTokenSet('token1'), false);
			await expect(async () => await gui.getTokenInfo('token1')).rejects.toThrow(
				"Missing required field 'tokens' in root"
			);

			assert.equal(stateUpdateCallback.mock.calls.length, 2);
			expect(stateUpdateCallback).toHaveBeenCalledWith(gui.serializeState());
		});

		it('should get network key', async () => {
			const networkKey = gui.getNetworkKey();
			assert.equal(networkKey, 'some-network');
		});
	});
});
