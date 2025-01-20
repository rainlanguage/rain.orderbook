import assert from 'assert';
import { getLocal } from 'mockttp';
import { describe, it, beforeEach, afterEach } from 'vitest';
import { Vault, VaultWithSubgraphName, Deposit } from '../../dist/types/js_api.js';
import { getVaults, getVault, getVaultBalanceChanges } from '../../dist/cjs/js_api.js';
import { VaultBalanceChange } from '../../dist/types/quote.js';

const vault1: Vault = {
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
const vault2: Vault = {
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
	beforeEach(() => mockServer.start(8083));
	afterEach(() => mockServer.stop());

	it('should fetch a single vault', async () => {
		await mockServer.forPost('/sg1').thenReply(200, JSON.stringify({ data: { vault: vault1 } }));

		try {
			const result: Vault = await getVault(mockServer.url + '/sg1', vault1.id);
			assert.equal(result.id, vault1.id);
		} catch (e) {
			console.log(e);
			assert.fail('expected to resolve, but failed');
		}
	});

	it('should fetch multiple vaults from different subgraphs', async () => {
		await mockServer.forPost('/sg1').thenReply(200, JSON.stringify({ data: { vaults: [vault1] } }));
		await mockServer.forPost('/sg2').thenReply(200, JSON.stringify({ data: { vaults: [vault2] } }));

		try {
			const result: VaultWithSubgraphName[] = await getVaults(
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
			assert.equal(result.length, 2);
			assert.equal(result[0].vault.id, vault1.id);
			assert.equal(result[0].subgraphName, 'network-one');
			assert.equal(result[1].vault.id, vault2.id);
			assert.equal(result[1].subgraphName, 'network-two');
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
			const result: Deposit[] = await getVaultBalanceChanges(mockServer.url + '/sg3', vault1.id, {
				page: 1,
				pageSize: 1
			});
			assert.equal(result[0].__typename, 'Deposit');
			assert.equal(result[0].amount, '5000000000000000000');
			assert.equal(result[0].newVaultBalance, '5000000000000000000');
			assert.equal(result[0].oldVaultBalance, '0');
		} catch (e) {
			console.log(e);
			assert.fail('expected to resolve, but failed');
		}
	});

	it('should handle errors when fetching vault balance changes', async () => {
		await mockServer.forPost('/sg1').thenReply(500, 'Internal Server Error');

		try {
			await getVaultBalanceChanges(mockServer.url + '/sg1', 'vault1', { page: 1, pageSize: 10 });
			assert.fail('expected to reject, but resolved');
		} catch (e) {
			assert.ok(e);
		}
	});
});
