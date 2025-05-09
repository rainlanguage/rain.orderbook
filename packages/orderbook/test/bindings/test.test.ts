import assert from 'assert';
import { describe, it } from 'vitest';
import { getOrderHash, OrderV3 } from '../../dist/cjs';
import { expect } from 'chai';

describe('Rain Orderbook Bindings Package Bindgen Tests', async function () {
	it('should get correct order hash', async () => {
		const order: OrderV3 = {
			owner: '0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba',
			evaluable: {
				interpreter: '0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba',
				store: '0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba',
				bytecode: '0x0102'
			},
			validInputs: [
				{
					token: '0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba',
					decimals: 7,
					vaultId: '0'
				}
			],
			validOutputs: [
				{
					token: '0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba',
					decimals: 18,
					vaultId: '0x1234'
				}
			],
			nonce: '0x0000000000000000000000000000000000000000000000000000000000000002'
		};
		const result = getOrderHash(order);
		if (!result.value) expect.fail('expected to resolve, but did not');
		const expected = '0xf4058d50e798f18a048097265fe67fe2e8619f337b9377a7620bb87fc2f52721';
		assert.equal(result.value, expected);
	});

	it('should error bad address length', async () => {
		try {
			const order: OrderV3 = {
				owner: '0xF14E09601A47552De6aBd3A0B165607FaFd2B5', // bad length
				evaluable: {
					interpreter: '0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba',
					store: '0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba',
					bytecode: '0x0102'
				},
				validInputs: [
					{
						token: '0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba',
						decimals: 7,
						vaultId: '0'
					}
				],
				validOutputs: [
					{
						token: '0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba',
						decimals: 18,
						vaultId: '0x1234'
					}
				],
				nonce: '0x0000000000000000000000000000000000000000000000000000000000000002'
			};
			const result = getOrderHash(order);
			if (!result.error) expect.fail('expected to error, but did not');
			assert.fail('expected to error, but resolved');
		} catch (error) {
			assert.ok(error instanceof Error);
			assert.ok(error.message.includes('Invalid string length'));
		}
	});

	it('should error bad vault id', async () => {
		try {
			const order: OrderV3 = {
				owner: '0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba',
				evaluable: {
					interpreter: '0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba',
					store: '0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba',
					bytecode: '0x0102'
				},
				validInputs: [
					{
						token: '0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba',
						decimals: 7,
						vaultId: 'qwe' // bad vault id
					}
				],
				validOutputs: [
					{
						token: '0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba',
						decimals: 18,
						vaultId: '0x1234'
					}
				],
				nonce: '0x0000000000000000000000000000000000000000000000000000000000000002'
			};
			const result = getOrderHash(order);
			if (!result.error) expect.fail('expected to error, but did not');
			assert.fail('expected to error, but resolved');
		} catch (error) {
			assert.ok(error instanceof Error);
			assert.ok(
				error.message.includes('invalid value: string "qwe", expected a 32 byte hex string')
			);
		}
	});

	it('should error bad nonce', async () => {
		try {
			const order: OrderV3 = {
				owner: '0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba',
				evaluable: {
					interpreter: '0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba',
					store: '0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba',
					bytecode: '0x0102'
				},
				validInputs: [
					{
						token: '0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba',
						decimals: 7,
						vaultId: '0'
					}
				],
				validOutputs: [
					{
						token: '0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba',
						decimals: 18,
						vaultId: '0x1234'
					}
				],
				nonce: '0x0000000000000000000000000000000000000000000000000000000000000efg' // bad nonce
			};
			const result = getOrderHash(order);
			if (!result.error) expect.fail('expected to error, but did not');
			assert.fail('expected to error, but resolved');
		} catch (error) {
			assert.ok(error instanceof Error);
			assert.ok(error.message.includes("Invalid character 'g' at position 63"));
		}
	});
});
