import assert from 'assert';
import { getLocal } from 'mockttp';
import { describe, it, beforeEach, afterEach } from 'vitest';
import { Vault, VaultWithSubgraphName } from '../../dist/types/js_api.js';
import { getVaults, getVault, getVaultBalanceChanges } from '../../dist/cjs/js_api.js';
import { VaultBalanceChangeUnwrapped } from '../../dist/types/subgraphTypes.js';

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

	it.only('should fetch vault balance changes', async () => {
		const mockBalanceChanges = [
			{
				typename: 'Withdrawal',
				amount: '-6948524',
				newVaultBalance: '0',
				oldVaultBalance: '6948524',
				vault: {
					id: '0xfd84f3de9ce3a0e95890ba2f63f1c7e63ba5428c24db9b7339bc32027591c1be',
					vault_id: '73706088221472674136813086676684919698244959074948023338794369231277216202278',
					token: {
						id: '0xfbda5f676cb37624f28265a144a48b0d6e87d3b6',
						address: '0xfbda5f676cb37624f28265a144a48b0d6e87d3b6',
						name: 'Bridged USDC (Stargate)',
						symbol: 'USDC.e',
						decimals: '6'
					}
				},
				timestamp: '1733419849',
				transaction: {
					id: '0x5986c2676205d589c7ce2b4578b4f30103241a7f5a4a36909df0dc6babce8447',
					from: '0x77199602114bdecb272ac9d5038d7e01cccec362',
					blockNumber: '34051837',
					timestamp: '1733419849'
				},
				orderbook: {
					id: '0xcee8cd002f151a536394e564b84076c41bbbcd4d'
				}
			}
		];

		await mockServer
			.forPost('/sg1')
			.thenReply(200, JSON.stringify({ data: { vaultBalanceChanges: mockBalanceChanges } }));

		try {
			const result: VaultBalanceChangeUnwrapped[] = await getVaultBalanceChanges(
				mockServer.url + '/sg2',
				vault1.id,
				{ page: 1, pageSize: 10 }
			);
			console.log(result);
			assert.equal(result.length, 1);
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
