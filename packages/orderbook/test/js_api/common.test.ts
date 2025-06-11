import { afterEach, beforeEach, describe, it } from 'vitest';
import { getAddOrderCalldata, getRemoveOrderCalldata, SgOrder } from '../../dist/cjs';
import { assert } from 'chai';
import { getLocal } from 'mockttp';

describe('Rain Orderbook Common Package Bindgen Tests', async function () {
	const mockServer = getLocal();
	beforeEach(() => mockServer.start(8080));
	afterEach(() => mockServer.stop());

	const dotrain = `
version: 1
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
    bindings:
      key: 10

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
#key !Test binding
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
:;
`;

	it('should get correct calldata', async () => {
		// mock calls
		// iInterpreter() call
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

		const result = await getAddOrderCalldata(dotrain, 'some-deployment');
		if (result.error) assert.fail('expected no error');
		assert.equal(result.value.length, 2314);
	});

	it('should throw undefined deployment error', async () => {
		const res = await getAddOrderCalldata(dotrain, 'some-other-deployment');
		if (!res.error) assert.fail('expected error');
		assert.equal(res.error.msg, 'Undefined deployment');
		assert.equal(
			res.error.readableMsg,
			'The specified deployment was not found in the .rain file.'
		);
	});

	it('should throw frontmatter missing deployment error', async () => {
		const dotrain = `
version: 1
deployers:
  some-deployer:
    test: test
---
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
:;
`;
		const res = await getAddOrderCalldata(dotrain, 'some-deployment');
		if (!res.error) assert.fail('expected error');
		assert.equal(res.error.msg, 'Undefined deployment');
		assert.equal(
			res.error.readableMsg,
			'The specified deployment was not found in the .rain file.'
		);
	});

	it('should get remove order calldata', async () => {
		let order: SgOrder = {
			id: '1',
			orderBytes:
				'0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000006171c21b2e553c59a64d1337211b77c367cefe5d00000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000001c000000000000000000000000000000000000000000000000000000000000002400000000000000000000000000000000000000000000000000000000000000001000000000000000000000000379b966dc6b117dd47b5fc5308534256a4ab1bcc0000000000000000000000006e4b01603edbda617002a077420e98c86595748e000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000950000000000000000000000000000000000000000000000000000000000000002ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0000000000000000000000000000000000000000000000000b1a2bc2ec5000000000000000000000000000000000000000000000000000000000000000000015020000000c020200020110000001100001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000050c5725949a6f0c72e6c4a641f24049a917db0cb000000000000000000000000000000000000000000000000000000000000001200000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001000000000000000000000000833589fcd6edb6e08f4c7c32d4f71b54bda0291300000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000001',
			orderHash: '',
			owner: '',
			active: true,
			inputs: [],
			outputs: [],
			meta: undefined,
			orderbook: {
				id: '1'
			},
			addEvents: [],
			timestampAdded: '0',
			trades: [],
			removeEvents: []
		};
		const res = await getRemoveOrderCalldata(order);
		if (res.error) assert.fail('Expected no error', res.error.msg);
		assert.equal(
			res.value,
			'0x8d7b6beb000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000003000000000000000000000000006171c21b2e553c59a64d1337211b77c367cefe5d00000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000001c000000000000000000000000000000000000000000000000000000000000002400000000000000000000000000000000000000000000000000000000000000001000000000000000000000000379b966dc6b117dd47b5fc5308534256a4ab1bcc0000000000000000000000006e4b01603edbda617002a077420e98c86595748e000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000950000000000000000000000000000000000000000000000000000000000000002ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0000000000000000000000000000000000000000000000000b1a2bc2ec5000000000000000000000000000000000000000000000000000000000000000000015020000000c020200020110000001100001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000050c5725949a6f0c72e6c4a641f24049a917db0cb000000000000000000000000000000000000000000000000000000000000001200000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001000000000000000000000000833589fcd6edb6e08f4c7c32d4f71b54bda02913000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000'
		);
	});
});
