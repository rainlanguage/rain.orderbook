import assert from 'assert';
import { getLocal } from 'mockttp';
import { describe, it, beforeEach, afterEach } from 'vitest';
import {
	SgTrade,
	SgOrder,
	OrderPerformance,
	SgOrderWithSubgraphName,
	OrderWithSortedVaults
} from '../../dist/types/js_api.js';
import {
	getOrders,
	getOrderByHash,
	getOrderTradesList,
	getOrderTradeDetail,
	getOrderTradesCount,
	getOrderPerformance
} from '../../dist/cjs/js_api.js';

const order1: SgOrder = {
	id: 'order1',
	orderBytes:
		'0x0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000012000000000000000000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000',
	orderHash: '0x1',
	owner: '0x0000000000000000000000000000000000000000',
	outputs: [
		{
			id: '0x0000000000000000000000000000000000000001',
			token: {
				id: '0x0000000000000000000000000000000000000000',
				address: '0x0000000000000000000000000000000000000000',
				name: 'T1',
				symbol: 'T1',
				decimals: '0'
			},
			balance: '0',
			vaultId: '0x1',
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
			id: '0x0000000000000000000000000000000000000002',
			token: {
				id: '0x0000000000000000000000000000000000000000',
				address: '0x0000000000000000000000000000000000000000',
				name: 'T2',
				symbol: 'T2',
				decimals: '0'
			},
			balance: '0',
			vaultId: '0x2',
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
} as unknown as SgOrder;

const order2 = {
	id: 'order2',
	orderBytes:
		'0x0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000012000000000000000000000000000000000000000000000000000000000000001a00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000',
	orderHash: '0x2',
	owner: '0x0000000000000000000000000000000000000000',
	outputs: [
		{
			id: '0x0000000000000000000000000000000000000000',
			token: {
				id: '0x0000000000000000000000000000000000000000',
				address: '0x0000000000000000000000000000000000000000',
				name: 'T1',
				symbol: 'T1',
				decimals: '0'
			},
			balance: '0',
			vaultId: '0',
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
				address: '0x0000000000000000000000000000000000000000',
				name: 'T2',
				symbol: 'T2',
				decimals: '0'
			},
			balance: '0',
			vaultId: '0',
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
} as unknown as SgOrder;

export const order3 = {
	id: 'order1',
	orderBytes:
		'0x0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000012000000000000000000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000',
	orderHash: '0x1',
	owner: '0x0000000000000000000000000000000000000000',
	outputs: [
		{
			id: '0x0000000000000000000000000000000000000000',
			token: {
				id: 'token-1',
				address: '0x1111111111111111111111111111111111111111',
				name: 'Token One',
				symbol: 'TK1',
				decimals: '18'
			},
			balance: '0',
			vaultId: '1',
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
				id: 'token-2',
				address: '0x2222222222222222222222222222222222222222',
				name: 'Token Two',
				symbol: 'TK2',
				decimals: '18'
			},
			balance: '0',
			vaultId: '2',
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
} as unknown as SgOrder;

const mockOrderTradesList: SgTrade[] = [
	{
		id: '0x07db8b3f3e7498f9d4d0e40b98f57c020d3d277516e86023a8200a20464d4894',
		timestamp: '1632000000',
		tradeEvent: {
			sender: '0x0000000000000000000000000000000000000000',
			transaction: {
				id: '0x0000000000000000000000000000000000000000',
				from: '0x0000000000000000000000000000000000000000',
				timestamp: '1632000000',
				blockNumber: '0'
			}
		},
		outputVaultBalanceChange: {
			amount: '-100000000000000000000',
			vault: {
				id: 'vault-1',
				vaultId: '1',
				token: {
					id: 'token-1',
					address: '0x1111111111111111111111111111111111111111',
					name: 'Token One',
					symbol: 'TK1',
					decimals: '18'
				}
			},
			id: 'output-change-1',
			__typename: 'TradeVaultBalanceChange',
			newVaultBalance: '900',
			oldVaultBalance: '1000',
			timestamp: '1632000000',
			transaction: {
				id: '0x0000000000000000000000000000000000000000',
				from: '0x0000000000000000000000000000000000000000',
				timestamp: '1632000000',
				blockNumber: '0'
			},
			orderbook: { id: 'orderbook-1' }
		},
		order: {
			id: order1.id,
			orderHash: '0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef'
		},
		inputVaultBalanceChange: {
			amount: '50000000000000000000',
			vault: {
				id: 'vault-2',
				vaultId: '2',
				token: {
					id: 'token-2',
					address: '0x2222222222222222222222222222222222222222',
					name: 'Token Two',
					symbol: 'TK2',
					decimals: '18'
				}
			},
			id: 'input-change-1',
			__typename: 'TradeVaultBalanceChange',
			newVaultBalance: '150',
			oldVaultBalance: '100',
			timestamp: '1632000000',
			transaction: {
				id: '0x0000000000000000000000000000000000000000',
				from: '0x0000000000000000000000000000000000000000',
				timestamp: '1632000000',
				blockNumber: '0'
			},
			orderbook: { id: 'orderbook-1' }
		},
		orderbook: {
			id: 'orderbook-1'
		}
	}
] as unknown as SgTrade[];

const mockTrade: SgTrade = {
	id: 'trade1',
	order: {
		id: 'order1',
		orderHash: '0x1'
	},
	tradeEvent: {
		sender: '0x0000000000000000000000000000000000000000',
		transaction: {
			id: '0x0000000000000000000000000000000000000000',
			from: '0x0000000000000000000000000000000000000000',
			blockNumber: '0',
			timestamp: '0'
		}
	},
	timestamp: '0',
	orderbook: {
		id: '0x0000000000000000000000000000000000000000'
	},
	outputVaultBalanceChange: {
		id: '0x0000000000000000000000000000000000000000',
		__typename: 'TradeVaultBalanceChange',
		amount: '-7',
		newVaultBalance: '93',
		oldVaultBalance: '100',
		vault: {
			id: '0x0000000000000000000000000000000000000000',
			vaultId: '1',
			token: {
				id: '0x0000000000000000000000000000000000000000',
				address: '0x0000000000000000000000000000000000000000',
				name: 'T1',
				symbol: 'T1',
				decimals: '18'
			}
		},
		timestamp: '0',
		transaction: {
			id: '0x0000000000000000000000000000000000000000',
			from: '0x0000000000000000000000000000000000000000',
			blockNumber: '0',
			timestamp: '0'
		},
		orderbook: {
			id: '0x0000000000000000000000000000000000000000'
		}
	},
	inputVaultBalanceChange: {
		id: '0x0000000000000000000000000000000000000000',
		__typename: 'TradeVaultBalanceChange',
		amount: '5',
		newVaultBalance: '105',
		oldVaultBalance: '100',
		vault: {
			id: '0x0000000000000000000000000000000000000000',
			vaultId: '2',
			token: {
				id: '0x0000000000000000000000000000000000000000',
				address: '0x0000000000000000000000000000000000000000',
				name: 'T2',
				symbol: 'T2',
				decimals: '6'
			}
		},
		timestamp: '0',
		transaction: {
			id: '0x0000000000000000000000000000000000000000',
			from: '0x0000000000000000000000000000000000000000',
			blockNumber: '0',
			timestamp: '0'
		},
		orderbook: {
			id: '0x0000000000000000000000000000000000000000'
		}
	}
} as unknown as SgTrade;

describe('Rain Orderbook JS API Package Bindgen Tests - SgOrder', async function () {
	const mockServer = getLocal();
	beforeEach(() => mockServer.start(8082));
	afterEach(() => mockServer.stop());

	it('should fetch a single order', async () => {
		await mockServer.forPost('/sg1').thenReply(200, JSON.stringify({ data: { orders: [order1] } }));

		try {
			const result: OrderWithSortedVaults = await getOrderByHash(
				mockServer.url + '/sg1',
				order1.orderHash
			);
			assert.equal(result.order.id, order1.id);
		} catch (e) {
			console.log(e);
			assert.fail('expected to resolve, but failed' + (e instanceof Error ? e.message : String(e)));
		}
	});

	it('should fetch multiple orders from different subgraphs', async () => {
		await mockServer.forPost('/sg1').thenReply(200, JSON.stringify({ data: { orders: [order1] } }));
		await mockServer.forPost('/sg2').thenReply(200, JSON.stringify({ data: { orders: [order2] } }));

		try {
			const result: SgOrderWithSubgraphName[] = await getOrders(
				[
					{ url: mockServer.url + '/sg1', name: 'network-one' },
					{ url: mockServer.url + '/sg2', name: 'network-two' }
				],
				{
					owners: [],
					active: undefined,
					orderHash: undefined
				},
				{
					page: 1,
					pageSize: 10
				}
			);
			assert.equal(result.length, 2);
			assert.equal(result[0].order.id, order1.id);
			assert.equal(result[0].subgraphName, 'network-one');
			assert.equal(result[1].order.id, order2.id);
			assert.equal(result[1].subgraphName, 'network-two');
		} catch (e) {
			console.log(e);
			assert.fail('expected to resolve, but failed' + (e instanceof Error ? e.message : String(e)));
		}
	});

	it('should fetch trades for a single order', async () => {
		await mockServer.forPost('/sg1').thenReply(
			200,
			JSON.stringify({
				data: {
					trades: mockOrderTradesList
				}
			})
		);

		try {
			const result = await getOrderTradesList(
				mockServer.url + '/sg1',
				order1.id,
				{
					page: 1,
					pageSize: 10
				},
				undefined,
				undefined
			);

			assert.ok(result, 'Result should exist');
			assert.equal(result.length, 1, 'Should have one trade');
			assert.equal(
				result[0].id,
				'0x07db8b3f3e7498f9d4d0e40b98f57c020d3d277516e86023a8200a20464d4894',
				'Trade ID should match'
			);
		} catch (e: unknown) {
			console.error('Test error:', e);
			assert.fail(
				'Expected to resolve, but failed: ' + (e instanceof Error ? e.message : String(e))
			);
		}
	});

	it('should fetch order trade detail', async () => {
		await mockServer.forPost('/sg1').thenReply(200, JSON.stringify({ data: { trade: mockTrade } }));

		try {
			const result: SgTrade = await getOrderTradeDetail(mockServer.url + '/sg1', mockTrade.id);
			assert.equal(result.id, mockTrade.id);
			assert.equal(result.order.id, mockTrade.order.id);
			assert.equal(
				result.outputVaultBalanceChange.amount,
				mockTrade.outputVaultBalanceChange.amount
			);
			assert.equal(result.inputVaultBalanceChange.amount, mockTrade.inputVaultBalanceChange.amount);
		} catch (e) {
			console.log(e);
			assert.fail(
				'expected to resolve, but failed' + +(e instanceof Error ? e.message : String(e))
			);
		}
	});

	it('should fetch trade count for a single order', async () => {
		await mockServer.forPost('/sg1').thenReply(
			200,
			JSON.stringify({
				data: {
					trades: mockOrderTradesList
				}
			})
		);

		await mockServer.forPost('/sg1').thenReply(
			200,
			JSON.stringify({
				data: {
					trades: []
				}
			})
		);

		try {
			const count = await getOrderTradesCount(
				mockServer.url + '/sg1',
				'0x07db8b3f3e7498f9d4d0e40b98f57c020d3d277516e86023a8200a20464d4894',
				undefined,
				undefined
			);

			assert.strictEqual(typeof count, 'number', 'Count should be a number');
			assert.strictEqual(count, 1, 'Should count one trade');
		} catch (e) {
			console.error('Test error:', e);
			if (e instanceof Error) {
				console.error('Error details:', e.stack);
			}
			assert.fail(
				'Expected to resolve, but failed: ' + (e instanceof Error ? e.message : String(e))
			);
		}
	});

	it('should measure order performance given an order id and subgraph', async () => {
		const mockServer = getLocal();
		mockServer.start(8088);
		await mockServer
			.forPost('/sg1')
			.once()
			.thenReply(200, JSON.stringify({ data: { order: order3 } }));
		await mockServer
			.forPost('/sg1')
			.once()
			.thenReply(
				200,
				JSON.stringify({
					data: {
						trades: mockOrderTradesList
					}
				})
			);
		await mockServer.forPost('/sg1').thenReply(
			200,
			JSON.stringify({
				data: {
					trades: []
				}
			})
		);

		const result = await getOrderPerformance(
			mockServer.url + '/sg1',
			'0x07db8b3f3e7498f9d4d0e40b98f57c020d3d277516e86023a8200a20464d4894',
			BigInt(1632000000),
			BigInt(1734571449)
		);
		const expected: OrderPerformance = {
			orderId: 'order1',
			orderHash: '0x1',
			orderbook: '0x0000000000000000000000000000000000000000',
			denominatedPerformance: {
				token: {
					id: 'token-2',
					address: '0x2222222222222222222222222222222222222222',
					name: 'Token Two',
					symbol: 'TK2',
					decimals: '18'
				},
				apy: '0x0',
				apyIsNeg: false,
				netVol: '0x0',
				netVolIsNeg: false,
				startingCapital: '0x258'
			},
			startTime: 1632000000,
			endTime: 1734571449,
			inputsVaults: [
				{
					id: '2',
					token: {
						id: 'token-2',
						address: '0x2222222222222222222222222222222222222222',
						name: 'Token Two',
						symbol: 'TK2',
						decimals: '18'
					},
					volDetails: {
						totalIn: '0x2b5e3af16b1880000',
						totalOut: '0x0',
						totalVol: '0x2b5e3af16b1880000',
						netVol: '0x2b5e3af16b1880000'
					},
					apyDetails: {
						startTime: 1632000000,
						endTime: 1734571449,
						netVol: '0x2b5e3af16b1880000',
						capital: '0x96',
						apy: '0x13bce241d361f7aa7687c05aa7a4e5',
						isNeg: false
					}
				}
			],
			outputsVaults: [
				{
					id: '1',
					token: {
						id: 'token-1',
						address: '0x1111111111111111111111111111111111111111',
						name: 'Token One',
						symbol: 'TK1',
						decimals: '18'
					},
					volDetails: {
						totalIn: '0x0',
						totalOut: '0x56bc75e2d63100000',
						totalVol: '0x56bc75e2d63100000',
						netVol: '0x56bc75e2d63100000'
					},
					apyDetails: {
						startTime: 1632000000,
						endTime: 1734571449,
						netVol: '0x56bc75e2d63100000',
						capital: '0x384',
						apy: '0x6944b6b4675fd38d22d401e37e1a1',
						isNeg: true
					}
				}
			]
		};
		mockServer.stop();
		assert.deepEqual(result, expected);
	});

	it('should return vaults sorted by inputs, outputs and inputs and outputs', async () => {
		const inputs = [
			{
				id: '0x0000000000000000000000000000000000000001',
				token: {
					id: '0x0000000000000000000000000000000000000000',
					address: '0x0000000000000000000000000000000000000000',
					name: 'T2',
					symbol: 'T2',
					decimals: '0'
				},
				balance: '0',
				vaultId: '0x1',
				owner: '0x0000000000000000000000000000000000000000',
				ordersAsOutput: [],
				ordersAsInput: [],
				balanceChanges: [],
				orderbook: {
					id: '0x0000000000000000000000000000000000000000'
				}
			},
			{
				id: '0x0000000000000000000000000000000000000003',
				token: {
					id: '0x0000000000000000000000000000000000000000',
					address: '0x0000000000000000000000000000000000000000',
					name: 'T3',
					symbol: 'T3',
					decimals: '0'
				},
				balance: '0',
				vaultId: '0x3',
				owner: '0x0000000000000000000000000000000000000000',
				ordersAsOutput: [],
				ordersAsInput: [],
				balanceChanges: [],
				orderbook: {
					id: '0x0000000000000000000000000000000000000000'
				}
			}
		];
		const outputs = [
			{
				id: '0x0000000000000000000000000000000000000002',
				token: {
					id: '0x0000000000000000000000000000000000000000',
					address: '0x0000000000000000000000000000000000000000',
					name: 'T2',
					symbol: 'T2',
					decimals: '0'
				},
				balance: '0',
				vaultId: '0x2',
				owner: '0x0000000000000000000000000000000000000000',
				ordersAsOutput: [],
				ordersAsInput: [],
				balanceChanges: [],
				orderbook: {
					id: '0x0000000000000000000000000000000000000000'
				}
			},
			{
				id: '0x0000000000000000000000000000000000000003',
				token: {
					id: '0x0000000000000000000000000000000000000000',
					address: '0x0000000000000000000000000000000000000000',
					name: 'T3',
					symbol: 'T3',
					decimals: '0'
				},
				balance: '0',
				vaultId: '0x3',
				owner: '0x0000000000000000000000000000000000000000',
				ordersAsOutput: [],
				ordersAsInput: [],
				balanceChanges: [],
				orderbook: {
					id: '0x0000000000000000000000000000000000000000'
				}
			}
		];
		await mockServer
			.forPost('/sg1')
			.once()
			.thenReply(200, JSON.stringify({ data: { orders: [{ ...order1, inputs, outputs }] } }));

		try {
			const result: OrderWithSortedVaults = await getOrderByHash(mockServer.url + '/sg1', order1.orderHash);

			const inputs = result.vaults.get('inputs');
			const outputs = result.vaults.get('outputs');
			const inputsOutputs = result.vaults.get('inputs_outputs');

			if (!inputs || !outputs || !inputsOutputs) {
				assert.fail('inputs, outputs or inputsOutputs should not be null');
			}

			assert.equal(inputs.length, 1);
			assert.equal(outputs.length, 1);
			assert.equal(inputsOutputs.length, 1);

			assert.equal(inputs[0].id, '0x0000000000000000000000000000000000000001');
			assert.equal(outputs[0].id, '0x0000000000000000000000000000000000000002');
			assert.equal(inputsOutputs[0].id, '0x0000000000000000000000000000000000000003');
		} catch (e) {
			console.log(e);
			assert.fail('expected to resolve, but failed' + (e instanceof Error ? e.message : String(e)));
		}
	});

	it('should fetch an order by orderHash', async () => {
		const mockOrder = {
			...order1,
			orderHash: '0xbf8075f73b0a6418d719e52189d59bf35a0949e5983b3edbbc0338c02ab17353'
		};
		await mockServer
			.forPost('/sg1')
			.thenReply(200, JSON.stringify({ data: { orders: [mockOrder] } }));

		try {
			const result: OrderWithSortedVaults = await getOrderByHash(
				mockServer.url + '/sg1',
				mockOrder.orderHash
			);

			assert.equal(result.order.orderHash, mockOrder.orderHash);
		} catch (e) {
			console.log(e);
			assert.fail('expected to resolve, but failed' + (e instanceof Error ? e.message : String(e)));
		}
	});
});
