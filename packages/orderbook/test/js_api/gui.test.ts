import assert from 'assert';
import { afterAll, beforeAll, beforeEach, describe, expect, it, Mock, vi } from 'vitest';
import {
	DotrainOrderGui,
	ApprovalCalldataResult,
	DeploymentTransactionArgs,
	DepositCalldataResult,
	GuiCfg,
	GuiDeploymentCfg,
	GuiFieldDefinitionCfg,
	GuiPresetCfg,
	GuiSelectTokensCfg,
	NameAndDescriptionCfg,
	TokenAllowance,
	TokenDeposit,
	TokenInfo,
	AllGuiConfig,
	WasmEncodedResult,
	FieldValue
} from '../../dist/cjs';
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
          default: some-default-value
        - binding: binding-2
          name: Field 2 name
          description: Field 2 description
          min: 100
          presets:
            - value: "99.2"
            - value: "582.1"
            - value: "648.239"
          show-custom-field: true
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
          default: 10
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
spec-version: 1
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
spec-version: 1
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
spec-version: 1
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
const dotrainForRemotes = `
spec-version: 1
gui:
  name: Test
  description: Fixed limit order strategy
  deployments:
    test-deployment:
      name: Test deployment
      description: Test description
      deposits:
        - token: token1
        - token: token2
      fields:
        - binding: binding-1
          name: Field 1 name
          default: some-default-value
    other-deployment:
      name: Test deployment
      description: Test description
      deposits:
        - token: token3
      fields:
        - binding: binding-1
          name: Field 1 name
          default: some-default-value
networks:
    some-network:
        rpc: http://localhost:8085/rpc-url
        chain-id: 999
        network-id: 999
        currency: ZZ
using-networks-from:
  chainid:
    url: http://localhost:8085/remote-networks
    format: chainid
subgraphs:
    some-sg: https://www.some-sg.com
    other-sg: https://www.other-sg.com
metaboards:
    test: https://metaboard.com
deployers:
    some-deployer:
        network: remote-network
        address: 0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba
    other-deployer:
        network: some-network
        address: 0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba
orderbooks:
    some-orderbook:
        address: 0xc95A5f8eFe14d7a20BD2E5BAFEC4E71f8Ce0B9A6
        network: remote-network
        subgraph: some-sg
    other-orderbook:
        address: 0xc95A5f8eFe14d7a20BD2E5BAFEC4E71f8Ce0B9A6
        network: some-network
        subgraph: other-sg
using-tokens-from: http://localhost:8085/remote-tokens
tokens:
    token1:
        network: remote-network
        address: 0xc2132d05d31c914a87c6611c10748aeb04b58e8f
        decimals: 6
        label: Token 1
        symbol: T1
    token2:
        network: remote-network
        address: 0x8f3cf7ad23cd3cadbd9735aff958023239c6a063
        decimals: 18
        label: Token 2
        symbol: T2
    token3:
        network: some-network
        address: 0xadf0000000000000000000000000000000000000
        decimals: 6
        label: Token 3
        symbol: T3
scenarios:
    some-scenario:
        deployer: some-deployer
    other-scenario:
        deployer: other-deployer
orders:
    some-order:
      inputs:
        - token: token1
      outputs:
        - token: token2
      deployer: some-deployer
      orderbook: some-orderbook
    other-order:
      inputs:
        - token: token3
      outputs:
        - token: token3
      deployer: other-deployer
      orderbook: other-orderbook
deployments:
    test-deployment:
        scenario: some-scenario
        order: some-order
    other-deployment:
        scenario: other-scenario
        order: other-order
---
_: 10,
_: 20;
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

	const extractWasmEncodedData = <T>(result: WasmEncodedResult<T>, errorMessage?: string): T => {
		if (result.error) {
			assert.fail(errorMessage ?? result.error.msg);
		}

		if (typeof void 0 === typeof result.value) {
			return result.value as T;
		}

		return result.value;
	};

	it('should return available deployments', async () => {
		const result = await DotrainOrderGui.getDeploymentKeys(dotrainWithGui);
		const deployments = extractWasmEncodedData<string[]>(result);
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

		const gui = new DotrainOrderGui();
		let result = await gui.chooseDeployment(dotrainWithGui, 'some-deployment');
		extractWasmEncodedData(result);

		const guiConfig = extractWasmEncodedData<GuiCfg>(gui.getGuiConfig());
		assert.equal(guiConfig.name, 'Fixed limit');
		assert.equal(guiConfig.description, 'Fixed limit order strategy');
	});

	it('should initialize gui object with state update callback', async () => {
		const stateUpdateCallback = vi.fn();

		const gui = new DotrainOrderGui();
		const result = await gui.chooseDeployment(
			dotrainWithGui,
			'some-deployment',
			stateUpdateCallback
		);
		extractWasmEncodedData(result);

		gui.executeStateUpdateCallback();
		assert.equal(stateUpdateCallback.mock.calls.length, 1);
	});

	it('should get strategy details', async () => {
		const result = await DotrainOrderGui.getStrategyDetails(dotrainWithGui);
		const strategyDetails = extractWasmEncodedData<NameAndDescriptionCfg>(result);
		assert.equal(strategyDetails.name, 'Fixed limit');
		assert.equal(strategyDetails.description, 'Fixed limit order strategy');
		assert.equal(strategyDetails.short_description, 'Buy WETH with USDC on Base.');
	});

	it('should get deployment details', async () => {
		const result = await DotrainOrderGui.getDeploymentDetails(dotrainWithGui);
		const deploymentDetails = extractWasmEncodedData<Map<string, NameAndDescriptionCfg>>(result);
		const entries = Array.from(deploymentDetails.entries());
		assert.equal(entries[0][0], 'other-deployment');
		assert.equal(entries[0][1].name, 'Test test');
		assert.equal(entries[0][1].description, 'Test test test');
		assert.equal(entries[1][0], 'some-deployment');
		assert.equal(entries[1][1].name, 'Buy WETH with USDC on Base.');
		assert.equal(entries[1][1].description, 'Buy WETH with USDC for fixed price on Base network.');
	});

	it('should get deployment detail', async () => {
		const result = await DotrainOrderGui.getDeploymentDetail(dotrainWithGui, 'other-deployment');
		const deploymentDetail = extractWasmEncodedData<NameAndDescriptionCfg>(result);
		assert.equal(deploymentDetail.name, 'Test test');
		assert.equal(deploymentDetail.description, 'Test test test');
	});

	it('should get current deployment details', async () => {
		const gui = new DotrainOrderGui();
		mockServer
			.forPost('/rpc-url')
			.withBodyIncluding('0x82ad56cb')
			.thenSendJsonRpcResult(
				'0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000007546f6b656e203100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000025431000000000000000000000000000000000000000000000000000000000000'
			);
		await gui.chooseDeployment(dotrainWithGui, 'some-deployment');

		let deploymentDetail = extractWasmEncodedData<NameAndDescriptionCfg>(
			gui.getCurrentDeploymentDetails()
		);

		assert.equal(deploymentDetail.name, 'Buy WETH with USDC on Base.');
		assert.equal(
			deploymentDetail.description,
			'Buy WETH with USDC for fixed price on Base network.'
		);
		assert.equal(deploymentDetail.short_description, 'Buy WETH with USDC on Base.');
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
		const gui = new DotrainOrderGui();
		let result = await gui.chooseDeployment(dotrainWithGui, 'other-deployment');
		extractWasmEncodedData(result);

		const token1TokenInfo = extractWasmEncodedData<TokenInfo>(await gui.getTokenInfo('token1'));
		const token2TokenInfo = extractWasmEncodedData<TokenInfo>(await gui.getTokenInfo('token2'));

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
		const gui = new DotrainOrderGui();
		let result = await gui.chooseDeployment(dotrainWithGui, 'other-deployment');
		extractWasmEncodedData(result);

		const allTokenInfos = extractWasmEncodedData<TokenInfo[]>(await gui.getAllTokenInfos());

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
		let gui = new DotrainOrderGui();
		let stateUpdateCallback: Mock;
		beforeEach(async () => {
			stateUpdateCallback = vi.fn();
			mockServer
				.forPost('/rpc-url')
				.withBodyIncluding('0x82ad56cb')
				.thenSendJsonRpcResult(
					'0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000007546f6b656e203100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000025431000000000000000000000000000000000000000000000000000000000000'
				);
			const result = await gui.chooseDeployment(
				dotrainWithGui,
				'some-deployment',
				stateUpdateCallback
			);
			extractWasmEncodedData(result);
		});

		it('should add deposit', async () => {
			assert.equal(extractWasmEncodedData<boolean>(gui.hasAnyDeposit()), false);

			gui.saveDeposit('token1', '50.6');
			const deposits = extractWasmEncodedData<TokenDeposit[]>(gui.getDeposits());
			assert.equal(deposits.length, 1);

			assert.equal(extractWasmEncodedData<boolean>(gui.hasAnyDeposit()), true);

			assert.equal(stateUpdateCallback.mock.calls.length, 1);
			expect(stateUpdateCallback).toHaveBeenCalledWith(
				extractWasmEncodedData(gui.serializeState())
			);
		});

		it('should update deposit', async () => {
			gui.saveDeposit('token1', '50.6');
			gui.saveDeposit('token1', '100.6');
			const deposits = extractWasmEncodedData<TokenDeposit[]>(gui.getDeposits());
			assert.equal(deposits.length, 1);
			assert.equal(deposits[0].amount, '100.6');

			assert.equal(stateUpdateCallback.mock.calls.length, 2);
			expect(stateUpdateCallback).toHaveBeenCalledWith(
				extractWasmEncodedData(gui.serializeState())
			);
		});

		it('should throw error if deposit token is not found in gui config', () => {
			const result = gui.getDepositPresets('token3');
			if (!result.error) expect.fail('Expected error');
			expect(result.error.msg).toBe('Deposit token not found in gui config: token3');
			expect(result.error.readableMsg).toBe(
				"The deposit token 'token3' was not found in the YAML configuration."
			);
		});

		it('should remove deposit', async () => {
			gui.saveDeposit('token1', '50.6');
			const deposits = extractWasmEncodedData<TokenDeposit[]>(gui.getDeposits());
			assert.equal(deposits.length, 1);

			gui.removeDeposit('token1');
			const depositsAfterRemove = extractWasmEncodedData<TokenDeposit[]>(gui.getDeposits());
			assert.equal(depositsAfterRemove.length, 0);

			gui.saveDeposit('token1', '50.6');
			assert.equal(extractWasmEncodedData<TokenDeposit[]>(gui.getDeposits()).length, 1);
			gui.saveDeposit('token1', '');
			assert.equal(extractWasmEncodedData<TokenDeposit[]>(gui.getDeposits()).length, 0);

			assert.equal(stateUpdateCallback.mock.calls.length, 4);
			expect(stateUpdateCallback).toHaveBeenCalledWith(
				extractWasmEncodedData(gui.serializeState())
			);
		});

		it('should get deposit presets', async () => {
			const presets = extractWasmEncodedData<string[]>(gui.getDepositPresets('token1'));
			assert.equal(presets.length, 5);
			assert.equal(presets[0], '0');
			assert.equal(presets[1], '10');
			assert.equal(presets[2], '100');
			assert.equal(presets[3], '1000');
			assert.equal(presets[4], '10000');
		});

		it('should throw error if deposit token is not found in gui config', () => {
			const result = gui.getDepositPresets('token2');
			if (!result.error) expect.fail('Expected error');
			expect(result.error.msg).toBe('Deposit token not found in gui config: token2');
			expect(result.error.readableMsg).toBe(
				"The deposit token 'token2' was not found in the YAML configuration."
			);
		});
	});

	describe('field value tests', async () => {
		let gui = new DotrainOrderGui();
		let stateUpdateCallback: Mock;
		beforeEach(async () => {
			stateUpdateCallback = vi.fn();
			mockServer
				.forPost('/rpc-url')
				.withBodyIncluding('0x82ad56cb')
				.thenSendJsonRpcResult(
					'0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000007546f6b656e203100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000025431000000000000000000000000000000000000000000000000000000000000'
				);
			const result = await gui.chooseDeployment(
				dotrainWithGui,
				'some-deployment',
				stateUpdateCallback
			);
			extractWasmEncodedData(result);
		});

		it('should save the field value as presets', async () => {
			const allFieldDefinitions = extractWasmEncodedData<GuiFieldDefinitionCfg[]>(
				gui.getAllFieldDefinitions()
			);
			gui.saveFieldValue('binding-1', allFieldDefinitions[0].presets?.[0]?.value || '');
			assert.deepEqual(extractWasmEncodedData<FieldValue>(gui.getFieldValue('binding-1')), {
				binding: 'binding-1',
				value: allFieldDefinitions[0].presets?.[0]?.value || '',
				is_preset: true
			});
			gui.saveFieldValue('binding-1', allFieldDefinitions[0].presets?.[1]?.value || '');
			assert.deepEqual(extractWasmEncodedData<FieldValue>(gui.getFieldValue('binding-1')), {
				binding: 'binding-1',
				value: allFieldDefinitions[0].presets?.[1]?.value || '',
				is_preset: true
			});
			gui.saveFieldValue('binding-1', allFieldDefinitions[0].presets?.[2]?.value || '');
			assert.deepEqual(extractWasmEncodedData<FieldValue>(gui.getFieldValue('binding-1')), {
				binding: 'binding-1',
				value: allFieldDefinitions[0].presets?.[2]?.value || '',
				is_preset: true
			});

			assert.equal(stateUpdateCallback.mock.calls.length, 3);
			expect(stateUpdateCallback).toHaveBeenCalledWith(
				extractWasmEncodedData(gui.serializeState())
			);
		});

		it('should save field value as custom values', async () => {
			gui.saveFieldValues([
				{
					binding: 'binding-1',
					value: '0x1234567890abcdef1234567890abcdef12345678'
				},
				{
					binding: 'binding-2',
					value: '100'
				}
			]);
			gui.saveFieldValues([
				{
					binding: 'binding-1',
					value: 'some-string'
				},
				{
					binding: 'binding-2',
					value: 'true'
				}
			]);
			const fieldValues = extractWasmEncodedData<FieldValue[]>(gui.getAllFieldValues());
			assert.equal(fieldValues.length, 2);
			assert.deepEqual(fieldValues[0], {
				binding: 'binding-1',
				value: 'some-string',
				is_preset: true
			});
			assert.deepEqual(fieldValues[1], {
				binding: 'binding-2',
				value: 'true',
				is_preset: false
			});

			assert.equal(stateUpdateCallback.mock.calls.length, 4);
			expect(stateUpdateCallback).toHaveBeenCalledWith(
				extractWasmEncodedData(gui.serializeState())
			);
		});

		it('should throw error during save if field binding is not found in field definitions', () => {
			const result = gui.saveFieldValue('binding-3', '1');
			if (!result.error) expect.fail('Expected error');
			expect(result.error.msg).toBe('Field binding not found: binding-3');
			expect(result.error.readableMsg).toBe(
				"The field binding 'binding-3' could not be found in the YAML configuration."
			);
		});

		it('should get field value', async () => {
			gui.saveFieldValue('binding-1', '0x1234567890abcdef1234567890abcdef12345678');
			let fieldValue = extractWasmEncodedData<FieldValue>(gui.getFieldValue('binding-1'));
			assert.deepEqual(fieldValue, {
				binding: 'binding-1',
				value: '0x1234567890abcdef1234567890abcdef12345678',
				is_preset: true
			});

			gui.saveFieldValue('binding-2', 'true');
			fieldValue = extractWasmEncodedData<FieldValue>(gui.getFieldValue('binding-2'));
			assert.deepEqual(fieldValue, {
				binding: 'binding-2',
				value: 'true',
				is_preset: false
			});

			gui.saveFieldValue('binding-1', 'some-string');
			fieldValue = extractWasmEncodedData<FieldValue>(gui.getFieldValue('binding-1'));
			assert.deepEqual(fieldValue, {
				binding: 'binding-1',
				value: 'some-string',
				is_preset: true
			});

			gui.saveFieldValue('binding-2', '100.5');
			fieldValue = extractWasmEncodedData<FieldValue>(gui.getFieldValue('binding-2'));
			assert.deepEqual(fieldValue, {
				binding: 'binding-2',
				value: '100.5',
				is_preset: false
			});
		});

		it('should throw error during get if field binding is not found', () => {
			const result = gui.getFieldValue('binding-3');
			if (!result.error) expect.fail('Expected error');
			expect(result.error.msg).toBe('Field binding not found: binding-3');
			expect(result.error.readableMsg).toBe(
				"The field binding 'binding-3' could not be found in the YAML configuration."
			);
		});

		it('should correctly filter field definitions', async () => {
			const allFieldDefinitions = extractWasmEncodedData<GuiFieldDefinitionCfg[]>(
				gui.getAllFieldDefinitions()
			);
			assert.equal(allFieldDefinitions.length, 2);

			const fieldDefinitionsWithoutDefaults = extractWasmEncodedData<GuiFieldDefinitionCfg[]>(
				gui.getAllFieldDefinitions(true)
			);
			assert.equal(fieldDefinitionsWithoutDefaults.length, 1);
			assert.equal(fieldDefinitionsWithoutDefaults[0].binding, 'binding-1');

			const fieldDefinitionsWithDefaults = extractWasmEncodedData<GuiFieldDefinitionCfg[]>(
				gui.getAllFieldDefinitions(false)
			);
			assert.equal(fieldDefinitionsWithDefaults.length, 1);
			assert.equal(fieldDefinitionsWithDefaults[0].binding, 'binding-2');
		});
	});

	describe('field definition tests', async () => {
		let gui = new DotrainOrderGui();
		beforeAll(async () => {
			mockServer
				.forPost('/rpc-url')
				.withBodyIncluding('0x82ad56cb')
				.thenSendJsonRpcResult(
					'0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000007546f6b656e203100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000025431000000000000000000000000000000000000000000000000000000000000'
				);
			const result = await gui.chooseDeployment(dotrainWithGui, 'some-deployment');
			extractWasmEncodedData(result);
		});

		it('should get field definition', async () => {
			const allFieldDefinitions = extractWasmEncodedData<GuiFieldDefinitionCfg[]>(
				gui.getAllFieldDefinitions()
			);
			assert.equal(allFieldDefinitions.length, 2);

			const fieldDefinition = extractWasmEncodedData<GuiFieldDefinitionCfg>(
				gui.getFieldDefinition('binding-1')
			);
			assert.equal(fieldDefinition.name, 'Field 1 name');
			assert.equal(fieldDefinition.description, 'Field 1 description');
			assert.equal(fieldDefinition.presets?.length, 3);
			assert.equal(fieldDefinition.default, 'some-default-value');
			assert.equal(fieldDefinition.showCustomField, undefined);

			let presets = fieldDefinition.presets as GuiPresetCfg[];
			assert.equal(presets[0].name, 'Preset 1');
			assert.equal(presets[0].value, '0x1234567890abcdef1234567890abcdef12345678');
			assert.equal(presets[1].name, 'Preset 2');
			assert.equal(presets[1].value, 'false');
			assert.equal(presets[2].name, 'Preset 3');
			assert.equal(presets[2].value, 'some-string');

			const fieldDefinition2 = extractWasmEncodedData<GuiFieldDefinitionCfg>(
				gui.getFieldDefinition('binding-2')
			);
			presets = fieldDefinition2.presets as GuiPresetCfg[];
			assert.equal(presets[0].value, '99.2');
			assert.equal(presets[1].value, '582.1');
			assert.equal(presets[2].value, '648.239');
			assert.equal(fieldDefinition2.default, undefined);
			assert.equal(fieldDefinition2.showCustomField, true);
		});

		it('should throw error during get if field binding is not found', () => {
			const result = gui.getFieldDefinition('binding-3');
			if (!result.error) expect.fail('Expected error');
			expect(result.error.msg).toBe('Field binding not found: binding-3');
			expect(result.error.readableMsg).toBe(
				"The field binding 'binding-3' could not be found in the YAML configuration."
			);
		});
	});

	describe('state management tests', async () => {
		let serializedState =
			'H4sIAAAAAAAA_7WOz0rDQBDGs1UqiAcRr4Lg1TWbDYmx1JMUqoiILCjeYrptaja7S7KJVB_Co1dfoPgEXr35POJNxFnbSK_OZb6Zb_78kPMTa5ANLw2-GcvBWI4Q9IizOu_Wsah4Czpt66iMS8-xsQw5IHthY4T-jixB9ghZfKxZWcBS5RxLbu5UkW1BLzVGd1xXqCQWqSpNJyJR4BY6wVUhHuxDZBWyr3usvwlyuCBQG62Azb4Ztj1kSVmTjjarf2Z97H5Md96709en4OXzqkUP3p4TtPGHlc5YqVVz53zfR7MqDMNdkBfn9b2-ZiN9RMjJ_llUVvJ0yGOWYWUmnvAvWZ33c3l7zHuH67CjTMoLPOBaqEnOpfkCsiQxOFUCAAA=';
		let dotrain3: string;
		let gui = new DotrainOrderGui();
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
			const result = await gui.chooseDeployment(dotrain3, 'other-deployment');
			extractWasmEncodedData(result);

			gui.saveFieldValue(
				'test-binding',
				extractWasmEncodedData<GuiFieldDefinitionCfg>(gui.getFieldDefinition('test-binding'))
					.presets?.[0].value || ''
			);
			gui.saveDeposit('token1', '50.6');
			gui.saveDeposit('token2', '100');
			gui.removeSelectToken('token1');
			await gui.saveSelectToken('token1', '0x6666666666666666666666666666666666666666');
			gui.setVaultId(true, 0, '666');
			gui.setVaultId(false, 0, '333');
		});

		it('should serialize gui state', async () => {
			const serialized = extractWasmEncodedData<string>(gui.serializeState());
			assert.equal(serialized, serializedState);
		});

		it('should deserialize gui state', async () => {
			const gui = new DotrainOrderGui();
			await gui.deserializeState(dotrain3, serializedState);

			const fieldValues = extractWasmEncodedData<FieldValue[]>(gui.getAllFieldValues());
			assert.equal(fieldValues.length, 1);
			assert.deepEqual(fieldValues[0], {
				binding: 'test-binding',
				value: 'test-value',
				is_preset: true
			});

			assert.equal(extractWasmEncodedData<boolean>(gui.isSelectTokenSet('token1')), true);
			assert.equal(extractWasmEncodedData<boolean>(gui.isSelectTokenSet('token2')), true);
			const deposits = extractWasmEncodedData<TokenDeposit[]>(gui.getDeposits());
			assert.equal(deposits.length, 2);
			assert.equal(deposits[0].token, 'token1');
			assert.equal(deposits[0].amount, '50.6');
			assert.equal(deposits[0].address, '0x6666666666666666666666666666666666666666');
			assert.equal(deposits[1].token, 'token2');
			assert.equal(deposits[1].amount, '100');
			assert.equal(deposits[1].address, '0x8f3cf7ad23cd3cadbd9735aff958023239c6a063');

			let result = gui.getCurrentDeployment();
			const guiDeployment = extractWasmEncodedData<GuiDeploymentCfg>(result);
			assert.equal(guiDeployment.deployment.order.inputs[0].vaultId, '0x29a');
			assert.equal(guiDeployment.deployment.order.outputs[0].vaultId, '0x14d');
		});

		it('should throw error if given dotrain is different', async () => {
			const gui = new DotrainOrderGui();
			let testDotrain = `${guiConfig}

${dotrainWithoutTokens}`;
			const result = await gui.deserializeState(testDotrain, serializedState);
			if (!result.error) expect.fail('Expected error');
			expect(result.error.msg).toBe('Deserialized dotrain mismatch');
			expect(result.error.readableMsg).toBe(
				'There was a mismatch in the dotrain configuration. Please check your YAML configuration for consistency.'
			);
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
			let result = await gui.chooseDeployment(testDotrain, 'other-deployment');
			extractWasmEncodedData(result);

			let deployment1 = extractWasmEncodedData<GuiDeploymentCfg>(gui.getCurrentDeployment());
			assert.equal(deployment1.deployment.order.inputs[0].vaultId, undefined);
			assert.equal(deployment1.deployment.order.outputs[0].vaultId, undefined);

			let serializedState = extractWasmEncodedData<string>(gui.serializeState());
			gui = new DotrainOrderGui();
			await gui.deserializeState(testDotrain, serializedState);

			let deployment2 = extractWasmEncodedData<GuiDeploymentCfg>(gui.getCurrentDeployment());
			assert.equal(deployment2.deployment.order.inputs[0].vaultId, undefined);
			assert.equal(deployment2.deployment.order.outputs[0].vaultId, undefined);
		});

		it('should get all the yaml fields', async () => {
			const gui = new DotrainOrderGui();
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
			dotrain3 = `${guiConfig}

${dotrain}`;
			await gui.chooseDeployment(dotrain3, 'some-deployment');

			const {
				fieldDefinitionsWithoutDefaults,
				fieldDefinitionsWithDefaults,
				deposits,
				orderInputs,
				orderOutputs
			} = extractWasmEncodedData<AllGuiConfig>(await gui.getAllGuiConfig());

			assert.equal(fieldDefinitionsWithoutDefaults.length, 1);
			assert.equal(fieldDefinitionsWithoutDefaults[0].binding, 'binding-2');

			assert.equal(fieldDefinitionsWithDefaults.length, 1);
			assert.equal(fieldDefinitionsWithDefaults[0].binding, 'binding-1');

			assert.equal(deposits.length, 1);
			assert.equal(deposits[0].token?.key, 'token1');

			assert.equal(orderInputs.length, 1);
			assert.equal(orderInputs[0].token?.key, 'token1');

			assert.equal(orderOutputs.length, 1);
			assert.equal(orderOutputs[0].token?.key, 'token2');
		});
	});

	describe('order operations tests', async () => {
		let gui = new DotrainOrderGui();

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
			const result = await gui.chooseDeployment(dotrain2, 'other-deployment');
			extractWasmEncodedData(result);
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

			const allowances = extractWasmEncodedData<TokenAllowance[]>(
				await gui.checkAllowances('0x1234567890abcdef1234567890abcdef12345678')
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

			const result = extractWasmEncodedData<ApprovalCalldataResult>(
				await gui.generateApprovalCalldatas('0x1234567890abcdef1234567890abcdef12345678')
			);

			// @ts-expect-error - result is valid
			assert.equal(result.Calldatas.length, 1);
			// @ts-expect-error - result is valid
			assert.equal(result.Calldatas[0].token, '0x8f3cf7ad23cd3cadbd9735aff958023239c6a063');
			assert.equal(
				// @ts-expect-error - result is valid
				result.Calldatas[0].calldata,
				'0x095ea7b3000000000000000000000000c95a5f8efe14d7a20bd2e5bafec4e71f8ce0b9a600000000000000000000000000000000000000000000010f0cf064dd59200000'
			);

			// Test no deposits case
			gui.removeDeposit('token1');
			gui.removeDeposit('token2');
			const emptyResult = extractWasmEncodedData<ApprovalCalldataResult>(
				await gui.generateApprovalCalldatas('0x1234567890abcdef1234567890abcdef12345678')
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

			const result = extractWasmEncodedData<DepositCalldataResult>(
				await gui.generateDepositCalldatas()
			);

			// @ts-expect-error - result is valid
			assert.equal(result.Calldatas.length, 1);
			assert.equal(
				// @ts-expect-error - result is valid
				result.Calldatas[0],
				'0x91337c0a0000000000000000000000008f3cf7ad23cd3cadbd9735aff958023239c6a063000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000010f0cf064dd5920000000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000000'
			);

			// Test no deposits case
			gui.removeDeposit('token1');
			gui.removeDeposit('token2');
			const emptyResult = extractWasmEncodedData<DepositCalldataResult>(
				await gui.generateDepositCalldatas()
			);
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

			gui.saveFieldValue('test-binding', '10');

			const addOrderCalldata = extractWasmEncodedData<string>(await gui.generateAddOrderCalldata());
			assert.equal(addOrderCalldata.length, 2314);

			let result = gui.getCurrentDeployment();
			const currentDeployment = extractWasmEncodedData<GuiDeploymentCfg>(result);
			assert.deepEqual(currentDeployment.deployment.scenario.bindings, {
				'test-binding': '10',
				'another-binding': '300'
			});
		});

		it('generates add order calldata without entering field value', async () => {
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

			const addOrderCalldata = extractWasmEncodedData<string>(await gui.generateAddOrderCalldata());
			assert.equal(addOrderCalldata.length, 2314);

			let result = gui.getCurrentDeployment();
			const currentDeployment = extractWasmEncodedData<GuiDeploymentCfg>(result);
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

			gui.saveFieldValue('test-binding', '0xbeef');

			const calldata = extractWasmEncodedData<string>(
				await gui.generateDepositAndAddOrderCalldatas()
			);
			assert.equal(calldata.length, 3146);

			let result = gui.getCurrentDeployment();
			const currentDeployment = extractWasmEncodedData<GuiDeploymentCfg>(result);
			assert.deepEqual(currentDeployment.deployment.scenario.bindings, {
				'test-binding': '0xbeef',
				'another-binding': '300'
			});
		});

		it('should generate multicalldata for deposit and add order with missing field value', async () => {
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

			gui.removeFieldValue('test-binding');
			assert.deepEqual(extractWasmEncodedData<FieldValue[]>(gui.getAllFieldValues()), []);

			gui.saveDeposit('token1', '1000');
			gui.saveDeposit('token2', '5000');

			const calldata = extractWasmEncodedData<string>(
				await gui.generateDepositAndAddOrderCalldatas()
			);
			assert.equal(calldata.length, 3146);

			let result = gui.getCurrentDeployment();
			const currentDeployment = extractWasmEncodedData<GuiDeploymentCfg>(result);
			assert.deepEqual(currentDeployment.deployment.scenario.bindings, {
				'test-binding': '10',
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
			const gui = new DotrainOrderGui();
			let result = await gui.chooseDeployment(testDotrain, 'other-deployment');
			extractWasmEncodedData(result);

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

			gui.saveFieldValue('test-binding', '0');

			const calldata = extractWasmEncodedData<string>(
				await gui.generateDepositAndAddOrderCalldatas()
			);
			assert.equal(calldata.length, 3146);

			const currentDeployment = extractWasmEncodedData<GuiDeploymentCfg>(
				gui.getCurrentDeployment()
			);
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
			const testGui = new DotrainOrderGui();
			let result = await testGui.chooseDeployment(testDotrain, 'other-deployment');
			extractWasmEncodedData(result);

			let result1 = await testGui.checkAllowances('0x1234567890abcdef1234567890abcdef12345678');
			if (result1.error) {
				expect(result1.error.msg).toBe('Token must be selected: token1');
				expect(result1.error.readableMsg).toBe("The token 'token1' must be selected to proceed.");
			} else expect.fail('Expected error');

			let result2 = await testGui.generateDepositCalldatas();
			if (result2.error) {
				expect(result2.error.msg).toBe('Token must be selected: token1');
				expect(result2.error.readableMsg).toBe("The token 'token1' must be selected to proceed.");
			} else expect.fail('Expected error');

			let result3 = await testGui.generateAddOrderCalldata();
			if (result3.error) {
				expect(result3.error.msg).toBe('Token must be selected: token1');
				expect(result3.error.readableMsg).toBe("The token 'token1' must be selected to proceed.");
			} else expect.fail('Expected error');

			let result4 = await testGui.generateDepositAndAddOrderCalldatas();
			if (result4.error) {
				expect(result4.error.msg).toBe('Token must be selected: token1');
				expect(result4.error.readableMsg).toBe("The token 'token1' must be selected to proceed.");
			} else expect.fail('Expected error');
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

			let guiConfig = `
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
			let testDotrain = `${guiConfig}

${dotrainWithoutVaultIds}`;
			const gui = new DotrainOrderGui();
			let result = await gui.chooseDeployment(testDotrain, 'other-deployment');
			extractWasmEncodedData(result);

			gui.saveDeposit('token1', '1000');
			gui.saveDeposit('token2', '5000');

			let result1 = await gui.generateAddOrderCalldata();
			if (result1.error) {
				expect(result1.error.msg).toBe('Missing field value: Test binding');
				expect(result1.error.readableMsg).toBe(
					"The value for field 'Test binding' is required but has not been set."
				);
			} else expect.fail('Expected error');

			let result2 = await gui.generateDepositAndAddOrderCalldatas();
			if (result2.error) {
				expect(result2.error.msg).toBe('Missing field value: Test binding');
				expect(result2.error.readableMsg).toBe(
					"The value for field 'Test binding' is required but has not been set."
				);
			} else expect.fail('Expected error');

			let missingFieldValues = extractWasmEncodedData<string[]>(gui.getMissingFieldValues());
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
			let result = await gui.chooseDeployment(testDotrain, 'other-deployment', stateUpdateCallback);
			extractWasmEncodedData(result);

			const currentDeployment = extractWasmEncodedData<GuiDeploymentCfg>(
				gui.getCurrentDeployment()
			);
			assert.equal(currentDeployment.deployment.order.inputs[0].vaultId, undefined);
			assert.equal(currentDeployment.deployment.order.outputs[0].vaultId, undefined);

			assert.equal(extractWasmEncodedData<boolean>(gui.hasAnyVaultId()), false);

			gui.setVaultId(true, 0, '0x123');

			assert.equal(extractWasmEncodedData<boolean>(gui.hasAnyVaultId()), true);

			assert.equal(
				extractWasmEncodedData<Map<string, (string | undefined)[]>>(gui.getVaultIds()).get(
					'input'
				)?.[0],
				'0x123'
			);
			assert.equal(
				extractWasmEncodedData<Map<string, (string | undefined)[]>>(gui.getVaultIds()).get(
					'output'
				)?.[0],
				undefined
			);

			gui.setVaultId(false, 0, '0x234');

			const newCurrentDeployment = extractWasmEncodedData<GuiDeploymentCfg>(
				gui.getCurrentDeployment()
			);
			assert.notEqual(newCurrentDeployment.deployment.order.inputs[0].vaultId, undefined);
			assert.notEqual(newCurrentDeployment.deployment.order.outputs[0].vaultId, undefined);
			assert.equal(newCurrentDeployment.deployment.order.inputs[0].vaultId, '0x123');
			assert.equal(newCurrentDeployment.deployment.order.outputs[0].vaultId, '0x234');

			const vaultIds = extractWasmEncodedData<Map<string, (string | undefined)[]>>(
				gui.getVaultIds()
			);
			assert.equal(vaultIds.get('input')?.[0], '0x123');
			assert.equal(vaultIds.get('output')?.[0], '0x234');

			gui.setVaultId(true, 0, undefined);
			assert.equal(
				extractWasmEncodedData<Map<string, (string | undefined)[]>>(gui.getVaultIds()).get(
					'input'
				)?.[0],
				undefined
			);

			gui.setVaultId(false, 0, '');
			assert.equal(
				extractWasmEncodedData<Map<string, (string | undefined)[]>>(gui.getVaultIds()).get(
					'output'
				)?.[0],
				undefined
			);

			result = gui.setVaultId(true, 0, 'test');
			if (!result.error) expect.fail('Expected error');
			expect(result.error.msg).toBe(
				"Invalid value for field 'vault-id': Failed to parse vault id in index '0' of inputs in order 'some-order'"
			);
			expect(result.error.readableMsg).toBe(
				"YAML configuration error: Invalid value for field 'vault-id': Failed to parse vault id in index '0' of inputs in order 'some-order'"
			);

			assert.equal(stateUpdateCallback.mock.calls.length, 4);
			expect(stateUpdateCallback).toHaveBeenCalledWith(
				extractWasmEncodedData(gui.serializeState())
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
			const calldatas = extractWasmEncodedData<DepositCalldataResult>(
				await gui.generateDepositCalldatas()
			);
			// @ts-expect-error - valid result
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
			gui.saveFieldValue('test-binding', '10');

			let result = extractWasmEncodedData<DeploymentTransactionArgs>(
				await gui.getDeploymentTransactionArgs('0x1234567890abcdef1234567890abcdef12345678')
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
			result = extractWasmEncodedData<DeploymentTransactionArgs>(
				await gui.getDeploymentTransactionArgs('0x1234567890abcdef1234567890abcdef12345678')
			);

			assert.equal(result.approvals.length, 0);
			assert.equal(result.deploymentCalldata.length, 2634);
			assert.equal(result.orderbookAddress, '0xc95a5f8efe14d7a20bd2e5bafec4e71f8ce0b9a6');
			assert.equal(result.chainId, 123);
		});
	});

	describe('select tokens tests', async () => {
		let gui = new DotrainOrderGui();
		let stateUpdateCallback: Mock;
		beforeEach(async () => {
			stateUpdateCallback = vi.fn();
			let dotrain3 = `
      ${guiConfig3}

      ${dotrainWithoutTokens}
      `;
			const result = await gui.chooseDeployment(dotrain3, 'other-deployment', stateUpdateCallback);
			extractWasmEncodedData(result);
		});

		it('should get select tokens', async () => {
			const selectTokens = extractWasmEncodedData<GuiSelectTokensCfg[]>(gui.getSelectTokens());
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
			const testGui = new DotrainOrderGui();
			let result = await testGui.chooseDeployment(dotrainWithGui, 'some-deployment');
			extractWasmEncodedData(result);

			assert.equal(
				extractWasmEncodedData<GuiSelectTokensCfg[]>(testGui.getSelectTokens()).length,
				0
			);

			result = await testGui.saveSelectToken('token1', '0x1');
			if (!result.error) expect.fail('Expected error');
			expect(result.error.msg).toBe('Select tokens not set');
			expect(result.error.readableMsg).toBe(
				'No tokens have been configured for selection. Please check your YAML configuration.'
			);
		});

		it('should throw error if token not found', async () => {
			const result = await gui.saveSelectToken('token3', '0x1');
			if (!result.error) expect.fail('Expected error');
			expect(result.error.msg).toBe('Token not found token3');
			expect(result.error.readableMsg).toBe(
				"The token 'token3' could not be found in the YAML configuration."
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

			assert.equal(extractWasmEncodedData<boolean>(gui.isSelectTokenSet('token1')), false);
			assert.equal(extractWasmEncodedData<boolean>(gui.isSelectTokenSet('token2')), false);
			assert.equal(extractWasmEncodedData<boolean>(gui.areAllTokensSelected()), false);

			let result = await gui.getTokenInfo('token1');
			if (!result.error) expect.fail('Expected error');
			expect(result.error.msg).toBe("Missing required field 'tokens' in root");
			expect(result.error.readableMsg).toBe(
				"YAML configuration error: Missing required field 'tokens' in root"
			);

			result = await gui.getTokenInfo('token2');
			if (!result.error) expect.fail('Expected error');
			expect(result.error.msg).toBe("Missing required field 'tokens' in root");
			expect(result.error.readableMsg).toBe(
				"YAML configuration error: Missing required field 'tokens' in root"
			);

			await gui.saveSelectToken('token1', '0x6666666666666666666666666666666666666666');
			await gui.saveSelectToken('token2', '0x8888888888888888888888888888888888888888');

			assert.equal(extractWasmEncodedData<boolean>(gui.isSelectTokenSet('token1')), true);
			assert.equal(extractWasmEncodedData<boolean>(gui.isSelectTokenSet('token2')), true);
			assert.equal(extractWasmEncodedData<boolean>(gui.areAllTokensSelected()), true);

			result = await gui.getTokenInfo('token1');
			const tokenInfo = extractWasmEncodedData<TokenInfo>(result);
			assert.equal(tokenInfo.name, 'Token 1');
			assert.equal(tokenInfo.symbol, 'T1');
			assert.equal(tokenInfo.decimals, 6);

			result = await gui.getTokenInfo('token2');
			const tokenInfo2 = extractWasmEncodedData<TokenInfo>(result);
			assert.equal(tokenInfo2.name, 'Teken 2');
			assert.equal(tokenInfo2.symbol, 'T2');
			assert.equal(tokenInfo2.decimals, 18);

			assert.equal(stateUpdateCallback.mock.calls.length, 2);
			expect(stateUpdateCallback).toHaveBeenCalledWith(
				extractWasmEncodedData(gui.serializeState())
			);
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
			assert.equal(extractWasmEncodedData<boolean>(gui.isSelectTokenSet('token1')), true);

			let result = await gui.getTokenInfo('token1');
			const tokenInfo1 = extractWasmEncodedData<TokenInfo>(result);
			assert.equal(tokenInfo1.name, 'Token 1');
			assert.equal(tokenInfo1.symbol, 'T1');
			assert.equal(tokenInfo1.decimals, 6);

			await gui.saveSelectToken('token1', '0x8888888888888888888888888888888888888888');
			assert.equal(extractWasmEncodedData<boolean>(gui.isSelectTokenSet('token1')), true);

			result = await gui.getTokenInfo('token1');
			const tokenInfo2 = extractWasmEncodedData<TokenInfo>(result);
			assert.equal(tokenInfo2.name, 'Teken 2');
			assert.equal(tokenInfo2.symbol, 'T2');
			assert.equal(tokenInfo2.decimals, 18);

			assert.equal(stateUpdateCallback.mock.calls.length, 2);
			expect(stateUpdateCallback).toHaveBeenCalledWith(
				extractWasmEncodedData(gui.serializeState())
			);
		});

		it('should remove select token', async () => {
			stateUpdateCallback = vi.fn();
			let dotrain3 = `
      ${guiConfig3}

      ${dotrainWithoutTokens}
      `;
			let result = await gui.chooseDeployment(dotrain3, 'other-deployment', stateUpdateCallback);
			extractWasmEncodedData(result);

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
			assert.equal(extractWasmEncodedData<boolean>(gui.isSelectTokenSet('token1')), true);

			const tokenInfo = extractWasmEncodedData<TokenInfo>(await gui.getTokenInfo('token1'));
			assert.equal(tokenInfo.name, 'Token 1');
			assert.equal(tokenInfo.symbol, 'T1');
			assert.equal(tokenInfo.decimals, 6);

			gui.removeSelectToken('token1');
			assert.equal(extractWasmEncodedData<boolean>(gui.isSelectTokenSet('token1')), false);

			let result1 = await gui.getTokenInfo('token1');
			if (!result1.error) expect.fail('Expected error');
			expect(result1.error.msg).toBe("Missing required field 'tokens' in root");
			expect(result1.error.readableMsg).toBe(
				"YAML configuration error: Missing required field 'tokens' in root"
			);

			assert.equal(stateUpdateCallback.mock.calls.length, 2);
			expect(stateUpdateCallback).toHaveBeenCalledWith(
				extractWasmEncodedData(gui.serializeState())
			);
		});

		it('should get network key', async () => {
			const networkKey = extractWasmEncodedData<string>(gui.getNetworkKey());
			assert.equal(networkKey, 'some-network');
		});
	});

	describe('remote network tests', () => {
		let gui: DotrainOrderGui;

		beforeEach(() => {
			gui = new DotrainOrderGui();
		});

		it('should fetch remote networks', async () => {
			mockServer
				.forGet('/remote-networks')
				.once()
				.thenJson(200, [
					{
						name: 'Remote',
						chain: 'remote-network',
						chainId: 123,
						rpc: ['http://localhost:8085/rpc-url'],
						networkId: 123,
						nativeCurrency: {
							name: 'Remote',
							symbol: 'RN',
							decimals: 18
						},
						infoURL: 'http://localhost:8085/info-url',
						shortName: 'remote-network'
					}
				]);
			mockServer
				.forGet('/remote-tokens')
				.once()
				.thenJson(200, {
					name: 'Remote',
					timestamp: '2021-01-01T00:00:00.000Z',
					keywords: [],
					version: {
						major: 1,
						minor: 0,
						patch: 0
					},
					tokens: [],
					logoUri: 'http://localhost.com'
				});

			await gui.chooseDeployment(dotrainForRemotes, 'test-deployment');
			assert.ok(gui.getCurrentDeployment());
		});

		it('should fail for same remote network key in response', async () => {
			mockServer
				.forGet('/remote-networks')
				.once()
				.thenJson(200, [
					{
						name: 'Remote',
						chain: 'remote-network',
						chainId: 123,
						rpc: ['http://localhost:8085/rpc-url'],
						networkId: 123,
						nativeCurrency: {
							name: 'Remote',
							symbol: 'RN',
							decimals: 18
						},
						infoURL: 'http://localhost:8085/info-url',
						shortName: 'remote-network'
					},
					{
						name: 'Remote',
						chain: 'remote-network',
						chainId: 123,
						rpc: ['http://localhost:8085/rpc-url'],
						networkId: 123,
						nativeCurrency: {
							name: 'Remote',
							symbol: 'RN',
							decimals: 18
						},
						infoURL: 'http://localhost:8085/info-url',
						shortName: 'remote-network'
					}
				]);

			const result = await gui.chooseDeployment(dotrainForRemotes, 'test-deployment');
			if (!result.error) expect.fail('Expected error');
			expect(result.error.msg).toBe(
				"Conflicting remote network in response, a network with key 'remote-network' already exists"
			);
			expect(result.error.readableMsg).toBe(
				"Order configuration error in YAML: Conflicting remote network in response, a network with key 'remote-network' already exists"
			);
		});

		it('should fail for duplicate network', async () => {
			mockServer
				.forGet('/remote-networks')
				.once()
				.thenJson(200, [
					{
						name: 'Remote',
						chain: 'remote-network',
						chainId: 123,
						rpc: ['http://localhost:8085/rpc-url'],
						networkId: 123,
						nativeCurrency: {
							name: 'Remote',
							symbol: 'RN',
							decimals: 18
						},
						infoURL: 'http://localhost:8085/info-url',
						shortName: 'remote-network'
					},
					{
						name: 'Some Network',
						chain: 'some-network',
						chainId: 999,
						rpc: ['http://localhost:8085/rpc-url'],
						networkId: 999,
						nativeCurrency: {
							name: 'Some Network',
							symbol: 'ZZ',
							decimals: 18
						},
						infoURL: 'http://localhost:8085/info-url',
						shortName: 'some-network'
					}
				]);
			mockServer
				.forGet('/remote-tokens')
				.once()
				.thenJson(200, {
					name: 'Remote',
					timestamp: '2021-01-01T00:00:00.000Z',
					keywords: [],
					version: {
						major: 1,
						minor: 0,
						patch: 0
					},
					tokens: [],
					logoUri: 'http://localhost.com'
				});

			const result = await gui.chooseDeployment(dotrainForRemotes, 'test-deployment');
			if (!result.error) expect.fail('Expected error');
			expect(result.error.msg).toBe('Remote network key shadowing: some-network');
			expect(result.error.readableMsg).toBe(
				'Order configuration error in YAML: Remote network key shadowing: some-network'
			);
		});
	});

	describe('remote tokens tests', () => {
		let gui: DotrainOrderGui;

		beforeEach(() => {
			gui = new DotrainOrderGui();
		});

		it('should fetch remote tokens', async () => {
			mockServer
				.forGet('/remote-networks')
				.once()
				.thenJson(200, [
					{
						name: 'Remote',
						chain: 'remote-network',
						chainId: 123,
						rpc: ['http://localhost:8085/rpc-url'],
						networkId: 123,
						nativeCurrency: {
							name: 'Remote',
							symbol: 'RN',
							decimals: 18
						},
						infoURL: 'http://localhost:8085/info-url',
						shortName: 'remote-network'
					}
				]);
			mockServer
				.forGet('/remote-tokens')
				.once()
				.thenJson(200, {
					name: 'Remote',
					timestamp: '2021-01-01T00:00:00.000Z',
					keywords: [],
					version: {
						major: 1,
						minor: 0,
						patch: 0
					},
					tokens: [
						{
							chainId: 123,
							address: '0x0000000000000000000000000000000000000000',
							name: 'Remote',
							symbol: 'RN',
							decimals: 18
						}
					],
					logoUri: 'http://localhost.com'
				});

			await gui.chooseDeployment(dotrainForRemotes, 'other-deployment');
			assert.ok(gui.getCurrentDeployment());
		});

		it('should fail for same remote token key in response', async () => {
			mockServer
				.forGet('/remote-networks')
				.once()
				.thenJson(200, [
					{
						name: 'Remote',
						chain: 'remote-network',
						chainId: 123,
						rpc: ['http://localhost:8085/rpc-url'],
						networkId: 123,
						nativeCurrency: {
							name: 'Remote',
							symbol: 'RN',
							decimals: 18
						},
						infoURL: 'http://localhost:8085/info-url',
						shortName: 'remote-network'
					}
				]);
			mockServer
				.forGet('/remote-tokens')
				.once()
				.thenJson(200, {
					name: 'Remote',
					timestamp: '2021-01-01T00:00:00.000Z',
					keywords: [],
					version: {
						major: 1,
						minor: 0,
						patch: 0
					},
					tokens: [
						{
							chainId: 123,
							address: '0x0000000000000000000000000000000000000000',
							name: 'Remote',
							symbol: 'RN',
							decimals: 18
						},
						{
							chainId: 123,
							address: '0x0000000000000000000000000000000000000000',
							name: 'Remote',
							symbol: 'RN',
							decimals: 18
						}
					],
					logoUri: 'http://localhost.com'
				});

			const result = await gui.chooseDeployment(dotrainForRemotes, 'other-deployment');
			if (!result.error) expect.fail('Expected error');
			expect(result.error.msg).toBe(
				"Conflicting remote token in response, a token with key 'remote' already exists"
			);
			expect(result.error.readableMsg).toBe(
				"Order configuration error in YAML: Conflicting remote token in response, a token with key 'remote' already exists"
			);
		});

		it('should fail for duplicate token', async () => {
			mockServer
				.forGet('/remote-networks')
				.once()
				.thenJson(200, [
					{
						name: 'Remote',
						chain: 'remote-network',
						chainId: 123,
						rpc: ['http://localhost:8085/rpc-url'],
						networkId: 123,
						nativeCurrency: {
							name: 'Remote',
							symbol: 'RN',
							decimals: 18
						},
						infoURL: 'http://localhost:8085/info-url',
						shortName: 'remote-network'
					}
				]);
			mockServer
				.forGet('/remote-tokens')
				.once()
				.thenJson(200, {
					name: 'Remote',
					timestamp: '2021-01-01T00:00:00.000Z',
					keywords: [],
					version: {
						major: 1,
						minor: 0,
						patch: 0
					},
					tokens: [
						{
							chainId: 123,
							address: '0x0000000000000000000000000000000000000000',
							name: 'Token1',
							symbol: 'RN',
							decimals: 18
						}
					],
					logoUri: 'http://localhost.com'
				});

			await gui.chooseDeployment(dotrainForRemotes, 'other-deployment');

			const result = await gui.getCurrentDeployment();
			if (!result.error) expect.fail('Expected error');
			expect(result.error.msg).toBe('Remote token key shadowing: token1');
			expect(result.error.readableMsg).toBe(
				'YAML configuration error: Remote token key shadowing: token1'
			);
		});
	});
});
