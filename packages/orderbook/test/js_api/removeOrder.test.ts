import assert from 'assert';
import { getLocal } from 'mockttp';
import { describe, it, beforeEach, afterEach } from 'vitest';
import { Transaction, RemoveOrderWithOrder } from '../../dist/types/js_api.js';
import { getTransactionRemoveOrders } from '../../dist/cjs/js_api.js';

const transaction1: Transaction = {
	id: '0x0da3659c0fd5258e962bf339afeaffddb06cc7a473802228b9586fe7503ed13a',
		from: '0xf08bcbce72f62c95dcb7c07dcb5ed26acfcfbc11',
		blockNumber: '37623990',
		timestamp: '1739815758'
};

const mockRemoveOrder: RemoveOrderWithOrder = {
	transaction: {
		id: '0x0da3659c0fd5258e962bf339afeaffddb06cc7a473802228b9586fe7503ed13a',
		from: '0xf08bcbce72f62c95dcb7c07dcb5ed26acfcfbc11',
		blockNumber: '37623990',
		timestamp: '1739815758'
	},
	order: {
		id: '0xd1639ec740f1fcfa7ca4aac827df554a03e88a36f13818d08ed77863f1be8177',
		orderBytes:
			'0x0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000f08bcbce72f62c95dcb7c07dcb5ed26acfcfbc1100000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000001e00000000000000000000000000000000000000000000000000000000000000260a815e3d897b3376629372a3afc69a320a611d52f3cf914335b9bc31021eec2990000000000000000000000005fb33d710f8b58de4c9fdec703b5c2487a5219d600000000000000000000000084c6e7f5a1e5dd89594cc25bef4722a1b8871ae6000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000ad00000000000000000000000000000000000000000000000000000000000000020000000000000000000000001d80c49bbbcd1c0911346656b529df9e5c2f783d0000000000000000000000000000000000000000000000000de0b6b3a7640000000000000000000000000000000000000000000000000000000000000000002d0200000024080500021810000001100001361100000110000101100000031000041e12000022130000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000001d80c49bbbcd1c0911346656b529df9e5c2f783d0000000000000000000000000000000000000000000000000000000000000012d863ddba8cdc7d7c413aa97726cfab247fe88490a271785ae7bfee35fdc4765600000000000000000000000000000000000000000000000000000000000000010000000000000000000000001d80c49bbbcd1c0911346656b529df9e5c2f783d0000000000000000000000000000000000000000000000000000000000000012d863ddba8cdc7d7c413aa97726cfab247fe88490a271785ae7bfee35fdc47656',
		orderHash: '0xb0d70b12a2ddb9fd96b5a5f20d778c4adf81d5c9c9b7755b7ca2f015545f9077',
		owner: '0xf08bcbce72f62c95dcb7c07dcb5ed26acfcfbc11',
		outputs: [
			{
				id: '0x62505c1dc17df48ee33b6365accc6f022e04a56ec326ed94ad42d6af2e1e2cc7',
				owner: '0xf08bcbce72f62c95dcb7c07dcb5ed26acfcfbc11',
				vaultId: '97876023468725745973349024037907513632727724269320958133054700715755104925270',
				balance: '0',
				token: {
					id: '0x1d80c49bbbcd1c0911346656b529df9e5c2f783d',
					address: '0x1d80c49bbbcd1c0911346656b529df9e5c2f783d',
					name: 'Wrapped Flare',
					symbol: 'WFLR',
					decimals: '18'
				},
				orderbook: {
					id: '0xcee8cd002f151a536394e564b84076c41bbbcd4d'
				},
				ordersAsOutput: [
					{
						id: '0xd1639ec740f1fcfa7ca4aac827df554a03e88a36f13818d08ed77863f1be8177',
						orderHash: '0xb0d70b12a2ddb9fd96b5a5f20d778c4adf81d5c9c9b7755b7ca2f015545f9077',
						active: false
					}
				],
				ordersAsInput: [
					{
						id: '0xd1639ec740f1fcfa7ca4aac827df554a03e88a36f13818d08ed77863f1be8177',
						orderHash: '0xb0d70b12a2ddb9fd96b5a5f20d778c4adf81d5c9c9b7755b7ca2f015545f9077',
						active: false
					}
				],
				balanceChanges: []
			}
		],
		inputs: [
			{
				id: '0x62505c1dc17df48ee33b6365accc6f022e04a56ec326ed94ad42d6af2e1e2cc7',
				owner: '0xf08bcbce72f62c95dcb7c07dcb5ed26acfcfbc11',
				vaultId: '97876023468725745973349024037907513632727724269320958133054700715755104925270',
				balance: '0',
				token: {
					id: '0x1d80c49bbbcd1c0911346656b529df9e5c2f783d',
					address: '0x1d80c49bbbcd1c0911346656b529df9e5c2f783d',
					name: 'Wrapped Flare',
					symbol: 'WFLR',
					decimals: '18'
				},
				orderbook: {
					id: '0xcee8cd002f151a536394e564b84076c41bbbcd4d'
				},
				ordersAsOutput: [
					{
						id: '0xd1639ec740f1fcfa7ca4aac827df554a03e88a36f13818d08ed77863f1be8177',
						orderHash: '0xb0d70b12a2ddb9fd96b5a5f20d778c4adf81d5c9c9b7755b7ca2f015545f9077',
						active: false
					}
				],
				ordersAsInput: [
					{
						id: '0xd1639ec740f1fcfa7ca4aac827df554a03e88a36f13818d08ed77863f1be8177',
						orderHash: '0xb0d70b12a2ddb9fd96b5a5f20d778c4adf81d5c9c9b7755b7ca2f015545f9077',
						active: false
					}
				],
				balanceChanges: []
			}
		],
		orderbook: {
			id: '0xcee8cd002f151a536394e564b84076c41bbbcd4d'
		},
		active: false,
		timestampAdded: '1739813495',
		meta: '0xff0a89c674ee7874a30058ed2f2a20302e2063616c63756c6174652d696f202a2f200a7573696e672d776f7264732d66726f6d203078466532343131434461313933443945346538334135633233344337466433323031303138383361430a6d61782d6f75747075743a206d61782d76616c756528292c0a696f3a206966280a2020657175616c2d746f280a202020206f75747075742d746f6b656e28290a202020203078316438306334396262626364316330393131333436363536623532396466396535633266373833640a2020290a2020310a2020696e762831290a293b0a0a2f2a20312e2068616e646c652d696f202a2f200a3a3b011bff13109e41336ff20278186170706c69636174696f6e2f6f637465742d73747265616d',
		addEvents: [
			{
				transaction: {
					id: '0xea3caf78e023487df10792f0a86d6988e165ff514870ef66f31a1c928874c982',
					from: '0xf08bcbce72f62c95dcb7c07dcb5ed26acfcfbc11',
					blockNumber: '37622839',
					timestamp: '1739813495'
				}
			}
		],
		trades: [],
		removeEvents: [
			{
				transaction: {
					id: '0x0da3659c0fd5258e962bf339afeaffddb06cc7a473802228b9586fe7503ed13a',
					from: '0xf08bcbce72f62c95dcb7c07dcb5ed26acfcfbc11',
					blockNumber: '37623990',
					timestamp: '1739815758'
				}
			}
		]
	}
};

const removeOrders: RemoveOrderWithOrder[] = [mockRemoveOrder];

describe('Rain Orderbook JS API Package Bindgen Tests - Remove Order', async function () {
	const mockServer = getLocal();
	beforeEach(() => mockServer.start(8099));
	afterEach(() => mockServer.stop());

	it('should fetch remove orders for a transaction', async () => {
		await mockServer
			.forPost('/sg1')
			.thenReply(200, JSON.stringify({ data: { removeOrders: removeOrders } }));
		try {
			const result: RemoveOrderWithOrder[] = await getTransactionRemoveOrders(
				mockServer.url,
				transaction1.id
			);
			console.log(JSON.stringify(result, null, 2));
			assert.equal(result[0].order.id, mockRemoveOrder.order.id);
		} catch (e) {
			assert.fail('expected to resolve, but failed' + (e instanceof Error ? e.message : String(e)));
		}
	});
});
