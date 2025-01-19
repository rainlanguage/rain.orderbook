import assert from 'assert';
import { describe, it } from 'vitest';
import { OrderV3 } from '../../dist/types/quote';
import { getOrderHash } from '../../dist/cjs/quote.js';

describe('Rain Orderbook Bindings Package Bindgen Tests', async function () {
	it('should get correct order hash', async () => {
		const order: OrderV3 = {
			owner: '0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba',
			evaluable: {
				interpreter: '0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba',
				store: '0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba',
				bytecode: Uint8Array.from([1, 2])
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
			nonce: '0x2'
		};
		const result = getOrderHash(order);
		const expected = '0xf4058d50e798f18a048097265fe67fe2e8619f337b9377a7620bb87fc2f52721';
		assert.equal(result, expected);
	});

	it('should error bad address length', async () => {
		try {
			const order: OrderV3 = {
				owner: '0xF14E09601A47552De6aBd3A0B165607FaFd2B5', // bad length
				evaluable: {
					interpreter: '0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba',
					store: '0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba',
					bytecode: Uint8Array.from([1, 2])
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
				nonce: '0x2'
			};
			getOrderHash(order);
			assert.fail('expected to error, but resolved');
		} catch (error) {
			assert.ok(error instanceof Error);
			assert.ok(error.message.includes('owner address, Invalid string length'));
		}
	});

	it('should error bad vault id', async () => {
		try {
			const order: OrderV3 = {
				owner: '0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba',
				evaluable: {
					interpreter: '0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba',
					store: '0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba',
					bytecode: Uint8Array.from([1, 2])
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
				nonce: '0x2'
			};
			getOrderHash(order);
			assert.fail('expected to error, but resolved');
		} catch (error) {
			assert.ok(error instanceof Error);
			assert.ok(error.message.includes('vault id, digit 26 is out of range for base 10'));
		}
	});

	it('should error bad nonce', async () => {
		try {
			const order: OrderV3 = {
				owner: '0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba',
				evaluable: {
					interpreter: '0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba',
					store: '0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba',
					bytecode: Uint8Array.from([1, 2])
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
				nonce: 'abcd' // bad nonce, doesnt have 0x
			};
			getOrderHash(order);
			assert.fail('expected to error, but resolved');
		} catch (error) {
			assert.ok(error instanceof Error);
			assert.ok(error.message.includes('nonce value, digit 10 is out of range for base 10'));
		}
	});
});
