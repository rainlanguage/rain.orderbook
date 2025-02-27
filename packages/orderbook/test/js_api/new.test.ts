import assert from 'assert';
import { describe, it } from 'vitest';
import { TestStruct } from '../../dist/cjs/js_api.js';
import { CustomError, TestStruct as TestStructType } from '../../dist/types/js_api';

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
const dotrainWithGui = `
${guiConfig}

${dotrain}
`;

describe('TestStruct', () => {
	it('should be able to call simpleFunction', () => {
		const result = TestStruct.simpleFunction();
		assert.equal(result.data, 'Hello, world!');
	});

	it('should be able to call errFunction', () => {
		let result = TestStruct.errFunction();
		if (result.data) {
			assert.fail('result.data should be undefined');
		}
		let error = {
			msg: 'JavaScript error: some error',
			readableMsg: 'Something went wrong: some error'
		} as CustomError;
		assert.deepEqual(result.error, error);
	});

	it('should be able to call simpleFunctionWithSelf', async () => {
		const testStruct = await TestStruct.newWithResult('beef');
		const result = testStruct.simpleFunctionWithSelf();
		assert.equal(result.data, 'Hello, beef!');
	});

	it('should be able to call errFunctionWithSelf', async () => {
		const testStruct: TestStructType = await TestStruct.newWithResult('beef');
		const result = testStruct.errFunctionWithSelf();
		if (result.data) {
			assert.fail('result.data should be undefined');
		}
		let error = {
			msg: 'Test error',
			readableMsg: 'An unexpected error occurred. Please try again.'
		} as CustomError;
		assert.deepEqual(result.error, error);
	});

	it('should be able to call simpleFunctionWithReturnType', () => {
		const result = TestStruct.simpleFunctionWithReturnType();
		assert.equal(result.data.field, 'Hello, world!');
	});

	it('should be able to call simpleFunctionWithReturnTypeWithSelf', () => {
		let testStruct = TestStruct.new('beef');
		const result = testStruct.simpleFunctionWithReturnTypeWithSelf();
		assert.equal(result.data.field, 'Hello, beef!');
	});

	it('should be able to call asyncFunction', async () => {
		const result = await TestStruct.asyncFunction();
		assert.equal(result.data, 123);
	});

	it('should be able to call asyncFunctionWithSelf', async () => {
		let testStruct = TestStruct.new('beef');
		const result = await testStruct.asyncFunctionWithSelf();
		assert.equal(result.data, 234);
	});

	it('should be able to call returnVec', () => {
		const result = TestStruct.returnVec();
		assert.deepEqual(result.data, [1, 2, 3]);
	});

	it('should be able to call returnHashMap', () => {
		const result = TestStruct.returnHashmap();
		assert.deepEqual(result.data, new Map([['key', 123]]));
	});

	it('should be able to call returnOption', () => {
		const result = TestStruct.returnOption();
		assert.equal(result.data, 123);
	});

	it('should be able to call returnOptionNone', () => {
		const result = TestStruct.returnOptionNone();
		assert.equal(result.data, undefined);
	});
});
