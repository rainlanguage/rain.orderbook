import assert from 'assert';
import { afterAll, beforeAll, beforeEach, describe, it } from 'vitest';
import { WasmEncodedResult, DotrainRegistry } from '../../dist/cjs';
import { getLocal } from 'mockttp';

const extractWasmEncodedData = <T>(result: WasmEncodedResult<T>, errorMessage?: string): T => {
	if (result.error) {
		assert.fail(errorMessage ?? result.error.msg);
	}
	if (typeof void 0 === typeof result.value) {
		return result.value as T;
	}
	return result.value;
};

const MOCK_SETTINGS_CONTENT = `
version: 3
networks:
  flare:
    rpcs: 
      - https://mainnet.flare.org
    chain-id: 14
    currency: ETH
  base:
    rpcs: 
      - https://mainnet.base.org
    chain-id: 8453
    currency: ETH
subgraphs:
  flare: https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/ob4-flare/0.8/gn
  base: https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/ob4-base/0.9/gn
metaboards:
  flare: https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/mb-flare-0x893BBFB7/0.1/gn
  base: https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/mb-base-0x59401C93/0.1/gn
orderbooks:
  flare:
    address: 0xCEe8Cd002F151A536394E564b84076c41bBBcD4d
    network: flare
    subgraph: flare
    deployment-block: 12345
  base:
    address: 0xd2938e7c9fe3597f78832ce780feb61945c377d7
    network: base
    subgraph: base
    deployment-block: 12345
deployers:
  flare:
    address: 0xE3989Ea7486c0F418C764e6c511e86f6E8830FAb
    network: flare
  base:
    address: 0xC1A14cE2fd58A3A2f99deCb8eDd866204eE07f8D
    network: base
tokens:
  token1:
    address: 0x4200000000000000000000000000000000000042
    network: flare
  token2:
    address: 0x4200000000000000000000000000000000000042
    network: base
`;

const MOCK_DOTRAIN_PREFIX = `
gui:
  name: Test gui
  description: Test description
  short-description: Test short description
  deployments:
    flare:
      name: Flare order name
      description: Flare order description
      deposits:
        - token: token1
          presets:
            - "0"
      fields:
        - binding: test-binding
          name: Test binding
          description: Test binding description
          presets:
            - value: "0xbeef"
          default: 10
    base:
      name: Base order name
      description: Base order description
      deposits:
        - token: token2
          presets:
            - "0"
      fields:
        - binding: test-binding
          name: Test binding
          description: Test binding description
          presets:
            - value: "0xbeef"
          default: 10
scenarios:
  flare:
    deployer: flare
    runs: 1
  base:
    deployer: base
    runs: 1
orders:
  flare:
    orderbook: flare
    inputs:
      - token: token1
    outputs:
      - token: token1
  base:
    orderbook: base
    inputs:
      - token: token2
    outputs:
      - token: token2
deployments:
  flare:
    scenario: flare
    order: flare
  base:
    scenario: base
    order: base`;

const FIRST_DOTRAIN_CONTENT = `
${MOCK_DOTRAIN_PREFIX}
---
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
:;`;

const SECOND_DOTRAIN_CONTENT = `
${MOCK_DOTRAIN_PREFIX}
---
#calculate-io
_ _: 1 1;
#handle-io
:;
#handle-add-order
:;`;

describe('Rain Orderbook JS API Package Bindgen Tests - Dotrain Registry', async function () {
	const mockServer = getLocal();
	beforeAll(async () => {
		await mockServer.start(8231);
	});
	afterAll(async () => {
		await mockServer.stop();
	});
	beforeEach(() => {
		mockServer.reset();
	});

	describe('DotrainRegistry Constructor', () => {
		it('should create registry and fetch all content successfully', async () => {
			const registryContent = `http://localhost:8231/settings.yaml
fixed-limit http://localhost:8231/fixed-limit.rain
auction-dca http://localhost:8231/auction-dca.rain`;

			await mockServer.forGet('/registry.txt').thenReply(200, registryContent);
			await mockServer.forGet('/settings.yaml').thenReply(200, MOCK_SETTINGS_CONTENT);
			await mockServer.forGet('/fixed-limit.rain').thenReply(200, FIRST_DOTRAIN_CONTENT);
			await mockServer.forGet('/auction-dca.rain').thenReply(200, SECOND_DOTRAIN_CONTENT);

			const result = await DotrainRegistry.new('http://localhost:8231/registry.txt');
			const registry = extractWasmEncodedData(result);

			assert.strictEqual(registry.registryUrl, 'http://localhost:8231/registry.txt');
			assert.strictEqual(registry.settingsUrl, 'http://localhost:8231/settings.yaml');
			assert.strictEqual(registry.registry, registryContent);
			assert.strictEqual(registry.settings, MOCK_SETTINGS_CONTENT);

			const orderUrls = registry.orderUrls;
			assert.strictEqual(orderUrls.size, 2);
			assert.strictEqual(orderUrls.get('fixed-limit'), 'http://localhost:8231/fixed-limit.rain');
			assert.strictEqual(orderUrls.get('auction-dca'), 'http://localhost:8231/auction-dca.rain');

			const orders = registry.orders;
			assert.strictEqual(orders.size, 2);
			assert(orders.has('fixed-limit'));
			assert(orders.has('auction-dca'));
		});

		it('should handle invalid registry format', async () => {
			const invalidContent = 'invalid format without proper structure';
			await mockServer.forGet('/invalid.txt').thenReply(200, invalidContent);

			const result = await DotrainRegistry.new('http://localhost:8231/invalid.txt');
			assert(result.error);
		});

		it('should handle empty registry file', async () => {
			await mockServer.forGet('/empty.txt').thenReply(200, '');

			const result = await DotrainRegistry.new('http://localhost:8231/empty.txt');
			assert(result.error);
			assert(result.error.readableMsg.includes('Invalid registry format'));
		});

		it('should handle settings fetch error', async () => {
			const registryContent =
				'http://localhost:8231/nonexistent-settings.yaml\norder1 http://localhost:8231/order1.rain';
			await mockServer.forGet('/registry.txt').thenReply(200, registryContent);
			await mockServer.forGet('/nonexistent-settings.yaml').thenReply(404);

			const result = await DotrainRegistry.new('http://localhost:8231/registry.txt');
			assert(result.error);
		});
	});

	describe('DotrainRegistry Order Management', () => {
		let registry: DotrainRegistry;

		beforeEach(async () => {
			const registryContent = `http://localhost:8231/settings.yaml
fixed-limit http://localhost:8231/fixed-limit.rain
auction-dca http://localhost:8231/auction-dca.rain`;

			await mockServer.forGet('/registry.txt').thenReply(200, registryContent);
			await mockServer.forGet('/settings.yaml').thenReply(200, MOCK_SETTINGS_CONTENT);
			await mockServer.forGet('/fixed-limit.rain').thenReply(200, FIRST_DOTRAIN_CONTENT);
			await mockServer.forGet('/auction-dca.rain').thenReply(200, SECOND_DOTRAIN_CONTENT);

			const result = await DotrainRegistry.new('http://localhost:8231/registry.txt');
			registry = extractWasmEncodedData(result);
		});

		it('should get order keys', () => {
			const keys = extractWasmEncodedData(registry.getOrderKeys());

			assert.strictEqual(keys.length, 2);
			assert(keys.includes('fixed-limit'));
			assert(keys.includes('auction-dca'));
		});

		it('should get all order details', () => {
			const orderDetails = extractWasmEncodedData(registry.getAllOrderDetails());

			assert.strictEqual(orderDetails.size, 2);
			assert(orderDetails.has('fixed-limit'));
			assert(orderDetails.has('auction-dca'));

			const fixedLimitDetails = orderDetails.get('fixed-limit');
			assert(fixedLimitDetails);
			assert.strictEqual(fixedLimitDetails.name, 'Test gui');
			assert.strictEqual(fixedLimitDetails.description, 'Test description');
			assert.strictEqual(fixedLimitDetails.short_description, 'Test short description');
		});

		it('should get deployment details for specific order', () => {
			const deploymentDetails = extractWasmEncodedData(
				registry.getDeploymentDetails('fixed-limit')
			);

			assert.strictEqual(deploymentDetails.size, 2);
			assert(deploymentDetails.has('flare'));
			assert(deploymentDetails.has('base'));

			const flareDetails = deploymentDetails.get('flare');
			assert(flareDetails);
			assert.strictEqual(flareDetails.name, 'Flare order name');
			assert.strictEqual(flareDetails.description, 'Flare order description');

			const baseDetails = deploymentDetails.get('base');
			assert(baseDetails);
			assert.strictEqual(baseDetails.name, 'Base order name');
			assert.strictEqual(baseDetails.description, 'Base order description');
		});

		it('should handle deployment details for non-existent order', () => {
			const result = registry.getDeploymentDetails('non-existent');
			assert(result.error);
			assert(result.error.readableMsg.includes("order key 'non-existent' was not found"));
		});
	});

	describe('DotrainRegistry GUI Creation', () => {
		let registry: DotrainRegistry;

		beforeEach(async () => {
			const registryContent = `http://localhost:8231/settings.yaml
fixed-limit http://localhost:8231/fixed-limit.rain`;

			await mockServer.forGet('/registry.txt').thenReply(200, registryContent);
			await mockServer.forGet('/settings.yaml').thenReply(200, MOCK_SETTINGS_CONTENT);
			await mockServer.forGet('/fixed-limit.rain').thenReply(200, FIRST_DOTRAIN_CONTENT);

			registry = extractWasmEncodedData(
				await DotrainRegistry.new('http://localhost:8231/registry.txt')
			);
		});

		it('should create GUI for valid order and deployment', async () => {
			const gui = extractWasmEncodedData(await registry.getGui('fixed-limit', 'flare', null));

			const currentDeployment = extractWasmEncodedData(gui.getCurrentDeployment());

			assert.strictEqual(currentDeployment.name, 'Flare order name');
			assert.strictEqual(currentDeployment.description, 'Flare order description');
		});

		it('should create GUI with state update callback', async () => {
			const stateCallback = () => {};

			const gui = extractWasmEncodedData(
				await registry.getGui('fixed-limit', 'base', stateCallback)
			);

			const currentDeployment = extractWasmEncodedData(gui.getCurrentDeployment());

			assert.strictEqual(currentDeployment.name, 'Base order name');
			assert.strictEqual(currentDeployment.description, 'Base order description');
		});

		it('should handle GUI creation for non-existent order', async () => {
			const result = await registry.getGui('non-existent', 'flare', null);
			assert(result.error);
			assert(result.error.readableMsg.includes("order key 'non-existent' was not found"));
		});
	});
});
