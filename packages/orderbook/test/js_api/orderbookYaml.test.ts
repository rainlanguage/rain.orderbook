import assert from 'assert';
import { describe, expect, it } from 'vitest';
import { OrderbookYaml, OrderbookCfg, WasmEncodedResult } from '../../dist/cjs';

const YAML_WITHOUT_ORDERBOOK = `
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
`;

const extractWasmEncodedData = <T>(result: WasmEncodedResult<T>, errorMessage?: string): T => {
	if (result.error) {
		assert.fail(errorMessage ?? result.error.msg);
	}

	if (typeof void 0 === typeof result.value) {
		return result.value as T;
	}

	return result.value;
};

describe('Rain Orderbook JS API Package Bindgen Tests - Settings', async function () {
	it('should create a new settings object', async function () {
		const orderbookYaml = new OrderbookYaml([YAML_WITHOUT_ORDERBOOK]);
		assert.ok(orderbookYaml);
	});

	describe('orderbook tests', async function () {
		it('should get the orderbook', async function () {
			const orderbookYaml = new OrderbookYaml([YAML_WITHOUT_ORDERBOOK]);

			const orderbook = extractWasmEncodedData<OrderbookCfg>(
				orderbookYaml.getOrderbookByAddress('0xc95A5f8eFe14d7a20BD2E5BAFEC4E71f8Ce0B9A6')
			);
			assert.equal(orderbook.address, '0xc95a5f8efe14d7a20bd2e5bafec4e71f8ce0b9a6');
			assert.equal(orderbook.network.chainId, 123);
			assert.equal(orderbook.subgraph.url, 'https://www.some-sg.com/');

			let result = orderbookYaml.getOrderbookByAddress('invalid-address');
			if (!result.error) expect.fail('Expected error');
			expect(result.error.msg).toBe('Invalid address: Odd number of digits');
			expect(result.error.readableMsg).toBe(
				'The provided address is invalid. Please ensure the address is in the correct hexadecimal format. Error: "Odd number of digits"'
			);

			result = orderbookYaml.getOrderbookByAddress('0x0000000000000000000000000000000000000000');
			if (!result.error) expect.fail('Expected error');
			expect(result.error.msg).toBe(
				"Orderbook yaml error: Key '0x0000000000000000000000000000000000000000' not found"
			);
			expect(result.error.readableMsg).toBe(
				'There was an error processing the YAML configuration. Please check the YAML file for any issues. Error: "Key \'0x0000000000000000000000000000000000000000\' not found"'
			);
		});
	});
});
