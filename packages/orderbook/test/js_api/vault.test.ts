import assert from 'assert';
import { getLocal } from 'mockttp';
import { describe, it, beforeEach, beforeAll, afterAll } from 'vitest';
import {
	getVaults,
	getVault,
	getVaultBalanceChanges,
	getVaultDepositCalldata,
	getVaultWithdrawCalldata,
	checkVaultAllowance,
	getVaultApprovalCalldata,
	SgVault,
	SgVaultWithSubgraphName,
	SgDeposit
} from '../../dist/cjs';

const vault1: SgVault = {
	id: 'vault1',
	owner: '0x0000000000000000000000000000000000000000',
	vaultId: '0x10',
	balance: '0x10',
	token: {
		id: 'token1',
		address: '0x0000000000000000000000000000000000000000',
		name: 'Token 1',
		symbol: 'TKN1',
		decimals: '18'
	},
	orderbook: {
		id: '0x0000000000000000000000000000000000000000'
	},
	ordersAsOutput: [],
	ordersAsInput: [],
	balanceChanges: []
};
const vault2: SgVault = {
	id: 'vault2',
	owner: '0x0000000000000000000000000000000000000000',
	vaultId: '0x20',
	balance: '0x20',
	token: {
		id: 'token2',
		address: '0x0000000000000000000000000000000000000000',
		name: 'Token 2',
		symbol: 'TKN2',
		decimals: '18'
	},
	orderbook: {
		id: '0x0000000000000000000000000000000000000000'
	},
	ordersAsOutput: [],
	ordersAsInput: [],
	balanceChanges: []
};

describe('Rain Orderbook JS API Package Bindgen Vault Tests', async function () {
	const mockServer = getLocal();
	beforeAll(async () => {
		await mockServer.start(8083);
	});
	afterAll(async () => {
		await mockServer.stop();
	});
	beforeEach(() => {
		mockServer.reset();
	});

	it('should fetch a single vault', async () => {
		await mockServer.forPost('/sg1').thenReply(200, JSON.stringify({ data: { vault: vault1 } }));

		try {
			const result = await getVault(mockServer.url + '/sg1', vault1.id);
			if (result.error) assert.fail('expected to resolve, but failed');
			assert.equal(result.value.id, vault1.id);
		} catch (e) {
			console.log(e);
			assert.fail('expected to resolve, but failed');
		}
	});

	it('should fetch multiple vaults from different subgraphs', async () => {
		await mockServer.forPost('/sg1').thenReply(200, JSON.stringify({ data: { vaults: [vault1] } }));
		await mockServer.forPost('/sg2').thenReply(200, JSON.stringify({ data: { vaults: [vault2] } }));

		try {
			const result = await getVaults(
				[
					{ url: mockServer.url + '/sg1', name: 'network-one' },
					{ url: mockServer.url + '/sg2', name: 'network-two' }
				],
				{
					owners: [],
					hideZeroBalance: false
				},
				{
					page: 1,
					pageSize: 10
				}
			);
			if (result.error) assert.fail('expected to resolve, but failed');
			const vaults = result.value;

			assert.equal(vaults.length, 2);
			assert.equal(vaults[0].vault.id, vault1.id);
			assert.equal(vaults[0].subgraphName, 'network-one');
			assert.equal(vaults[1].vault.id, vault2.id);
			assert.equal(vaults[1].subgraphName, 'network-two');
		} catch (e) {
			console.log(e);
			assert.fail('expected to resolve, but failed');
		}
	});

	it('should fetch vault balance changes', async () => {
		const mockVaultBalanceChanges = [
			{
				__typename: 'Deposit',
				amount: '5000000000000000000',
				newVaultBalance: '5000000000000000000',
				oldVaultBalance: '0',
				vault: {
					id: '0x166aeed725f0f3ef9fe62f2a9054035756d55e5560b17afa1ae439e9cd362902',
					vaultId: '1',
					token: {
						id: '0x1d80c49bbbcd1c0911346656b529df9e5c2f783d',
						address: '0x1d80c49bbbcd1c0911346656b529df9e5c2f783d',
						name: 'Wrapped Flare',
						symbol: 'WFLR',
						decimals: '18'
					}
				},
				timestamp: '1734054063',
				transaction: {
					id: '0x85857b5c6d0b277f9e971b6b45cab98720f90b8f24d65df020776d675b71fc22',
					from: '0x7177b9d00bb5dbcaaf069cc63190902763783b09',
					blockNumber: '34407047',
					timestamp: '1734054063'
				},
				orderbook: {
					id: '0xcee8cd002f151a536394e564b84076c41bbbcd4d'
				}
			}
		];

		await mockServer
			.forPost('/sg3')
			.once()
			.thenReply(200, JSON.stringify({ data: { vaultBalanceChanges: mockVaultBalanceChanges } }));

		try {
			const result = await getVaultBalanceChanges(mockServer.url + '/sg3', vault1.id, {
				page: 1,
				pageSize: 1
			});
			if (result.error) assert.fail('expected to resolve, but failed');
			const balanceChanges = result.value;

			assert.equal(balanceChanges[0].__typename, 'Deposit');
			assert.equal(balanceChanges[0].amount, '5000000000000000000');
			assert.equal(balanceChanges[0].newVaultBalance, '5000000000000000000');
			assert.equal(balanceChanges[0].oldVaultBalance, '0');
		} catch (e) {
			console.log(e);
			assert.fail('expected to resolve, but failed');
		}
	});

	it('should handle errors when fetching vault balance changes', async () => {
		await mockServer.forPost('/sg1').thenReply(500, 'Internal Server Error');

		const res = await getVaultBalanceChanges(mockServer.url + '/sg1', 'vault1', {
			page: 1,
			pageSize: 10
		});
		if (!res.error) assert.fail('expected to reject, but resolved');
	});

	const order = {
		id: 'order',
		orderBytes:
			'0x0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000012000000000000000000000000000000000000000000000000000000000000001a00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000',
		orderHash: '0x1',
		owner: '0x0000000000000000000000000000000000000000',
		outputs: [
			{
				id: '0x0000000000000000000000000000000000000000',
				token: {
					id: '0x0000000000000000000000000000000000000000',
					address: '0x1234567890123456789012345678901234567890',
					name: 'T1',
					symbol: 'T1',
					decimals: '0'
				},
				balance: '88888888888',
				vaultId: '0x2523',
				owner: '0x0000000000000000000000000000000000000000',
				ordersAsOutput: [],
				ordersAsInput: [],
				balanceChanges: [],
				orderbook: {
					id: '0x0000000000000000000000000000000000000000'
				}
			}
		],
		inputs: [
			{
				id: '0x0000000000000000000000000000000000000000',
				token: {
					id: '0x0000000000000000000000000000000000000000',
					address: '0x9876543210987654321098765432109876543210',
					name: 'T2',
					symbol: 'T2',
					decimals: '0'
				},
				balance: '999999999999999',
				vaultId: '0x100',
				owner: '0x0000000000000000000000000000000000000000',
				ordersAsOutput: [],
				ordersAsInput: [],
				balanceChanges: [],
				orderbook: {
					id: '0x0000000000000000000000000000000000000000'
				}
			}
		],
		active: true,
		addEvents: [
			{
				transaction: {
					blockNumber: '0',
					timestamp: '0',
					id: '0x0000000000000000000000000000000000000000',
					from: '0x0000000000000000000000000000000000000000'
				}
			}
		],
		meta: null,
		timestampAdded: '0',
		orderbook: {
			id: '0x0000000000000000000000000000000000000000'
		},
		trades: [],
		removeEvents: []
	};

	it('should get deposit calldata for a vault', async () => {
		await mockServer.forPost('/sg4').thenReply(200, JSON.stringify({ data: { order } }));

		const res = await getVaultDepositCalldata(vault1, '500');
		if (res.error) assert.fail('expected to resolve, but failed');
		assert.equal(res.value.length, 330);
	});

	it('should handle zero deposit amount', async () => {
		await mockServer.forPost('/sg4').thenReply(200, JSON.stringify({ data: { order } }));

		const res = await getVaultDepositCalldata(vault1, '0');
		if (!res.error) assert.fail('expected to reject, but resolved');
		assert.equal(res.error.msg, 'Invalid amount');
	});

	it('should throw error for invalid deposit amount', async () => {
		const res = await getVaultDepositCalldata(vault1, '-100');
		if (!res.error) assert.fail('expected to reject, but resolved');
		assert.equal(res.error.msg, 'invalid digit: -');
	});

	it('should get withdraw calldata for a vault', async () => {
		await mockServer.forPost('/sg4').thenReply(200, JSON.stringify({ data: { order } }));

		let res = await getVaultWithdrawCalldata(vault1, '500');
		if (res.error) assert.fail('expected to resolve, but failed');
		assert.equal(res.value.length, 330);

		res = await getVaultWithdrawCalldata(vault1, '0');
		if (!res.error) assert.fail('expected to reject, but resolved');
		assert.equal(res.error.msg, 'Invalid amount');
		assert.equal(res.error.readableMsg, 'Invalid amount');
	});

	it('should read allowance for a vault', async () => {
		await mockServer.forPost('/rpc').thenReply(
			200,
			JSON.stringify({
				jsonrpc: '2.0',
				id: 1,
				result: '0x0000000000000000000000000000000000000000000000000000000000000064'
			})
		);

		const res = await checkVaultAllowance([mockServer.url + '/rpc'], vault1);
		if (res.error) assert.fail('expected to resolve, but failed');
		assert.equal(res.value, '0x64');
	});

	it('should generate valid approval calldata with correct length', async () => {
		await mockServer.forPost('/rpc').thenReply(
			200,
			JSON.stringify({
				jsonrpc: '2.0',
				id: 1,
				result: '0x0000000000000000000000000000000000000000000000000000000000000064'
			})
		);

		const res = await getVaultApprovalCalldata([mockServer.url + '/rpc'], vault1, '600');
		if (res.error) assert.fail('expected to resolve, but failed');

		assert.ok(res.value.startsWith('0x'));
		assert.equal(res.value.length, 138);
	});
});
