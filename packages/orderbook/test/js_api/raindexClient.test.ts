import assert from 'assert';
import { afterAll, beforeAll, beforeEach, describe, it } from 'vitest';
import {
	WasmEncodedResult,
	RaindexClient,
	SgOrder,
	SgTrade,
	OrderPerformance,
	VaultVolume
} from '../../dist/cjs';
import { getLocal } from 'mockttp';

const YAML = `
networks:
    some-network:
        rpc: http://localhost:8230/rpc1
        chain-id: 1
        network-id: 1
        currency: ETH
    other-network:
        rpc: http://localhost:8230/rpc2
        chain-id: 2
        network-id: 2
        currency: ETH
subgraphs:
    some-sg: http://localhost:8230/sg1
    other-sg: http://localhost:8230/sg2
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
    other-orderbook:
        address: 0xc95A5f8eFe14d7a20BD2E5BAFEC4E71f8Ce0B9A6
        network: other-network
        subgraph: other-sg
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
			balance: '0x98723',
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
			balance: '0x7772',
			vaultId: '0x3',
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
			balance: '0x123',
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
			balance: '0x7772',
			vaultId: '0x3',
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
	meta: '0xff0a89c674ee7874a3005902252f2a20302e2063616c63756c6174652d696f202a2f200a7573696e672d776f7264732d66726f6d203078466532343131434461313933443945346538334135633233344337466433323031303138383361430a616d743a203130302c0a696f3a2063616c6c3c323e28293b0a0a2f2a20312e2068616e646c652d696f202a2f200a3a63616c6c3c333e28292c0a3a656e7375726528657175616c2d746f286f75747075742d7661756c742d64656372656173652829203130302920226d7573742074616b652066756c6c20616d6f756e7422293b0a0a2f2a20322e206765742d696f2d726174696f2d6e6f77202a2f200a656c61707365643a2063616c6c3c343e28292c0a696f3a2073617475726174696e672d73756228302e3031373733353620646976286d756c28656c61707365642073756228302e3031373733353620302e30313733383434292920363029293b0a0a2f2a20332e206f6e652d73686f74202a2f200a3a656e737572652869732d7a65726f286765742868617368286f726465722d68617368282920226861732d657865637574656422292929202268617320657865637574656422292c0a3a7365742868617368286f726465722d68617368282920226861732d657865637574656422292031293b0a0a2f2a20342e206765742d656c6170736564202a2f200a5f3a20737562286e6f772829206765742868617368286f726465722d68617368282920226465706c6f792d74696d65222929293b011bff13109e41336ff20278186170706c69636174696f6e2f6f637465742d73747265616d',
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

const extractWasmEncodedData = <T>(result: WasmEncodedResult<T>, errorMessage?: string): T => {
	if (result.error) {
		console.log(result.error);
		assert.fail(errorMessage ?? result.error.msg);
	}
	if (typeof void 0 === typeof result.value) {
		return result.value as T;
	}
	return result.value;
};

describe('Rain Orderbook JS API Package Bindgen Tests - Raindex Client', async function () {
	const mockServer = getLocal();
	beforeAll(async () => {
		await mockServer.start(8230);
	});
	afterAll(async () => {
		await mockServer.stop();
	});
	beforeEach(() => {
		mockServer.reset();
	});

	it('should create a new raindex client object', async function () {
		const raindexClient = RaindexClient.new([YAML]);
		assert.ok(extractWasmEncodedData(raindexClient));
	});

	describe('Orders', async function () {
		it('should get orders', async function () {
			await mockServer
				.forPost('/sg1')
				.thenReply(200, JSON.stringify({ data: { orders: [order1] } }));
			await mockServer
				.forPost('/sg2')
				.thenReply(200, JSON.stringify({ data: { orders: [order2] } }));

			const raindexClient = extractWasmEncodedData(RaindexClient.new([YAML]));

			let orders = extractWasmEncodedData(await raindexClient.getOrders());
			assert.equal(orders.length, 2);
			assert.equal(orders[0].id, order2.id);
			assert.equal(orders[1].id, order1.id);

			orders = extractWasmEncodedData(await raindexClient.getOrders(1));
			assert.equal(orders.length, 1);
			assert.equal(orders[0].id, order1.id);
		});

		it('should get order by hash', async function () {
			await mockServer
				.forPost('/sg1')
				.thenReply(200, JSON.stringify({ data: { orders: [order1] } }));
			const raindexClient = extractWasmEncodedData(RaindexClient.new([YAML]));

			const order = extractWasmEncodedData(await raindexClient.getOrderByHash(1, 'hash'));
			assert.equal(typeof order, 'object');
			assert.equal(order.id, order1.id);
			assert.equal(order.orderbook, order1.orderbook.id);
			assert.equal(order.owner, order1.owner);
			assert.equal(order.active, order1.active);
			assert.equal(order.timestampAdded, order1.timestampAdded);
			assert.equal(order.meta, order1.meta);
			assert.equal(
				order.rainlang,
				'/* 0. calculate-io */ \nusing-words-from 0xFe2411CDa193D9E4e83A5c234C7Fd320101883aC\namt: 100,\nio: call<2>();\n\n/* 1. handle-io */ \n:call<3>(),\n:ensure(equal-to(output-vault-decrease() 100) "must take full amount");\n\n/* 2. get-io-ratio-now */ \nelapsed: call<4>(),\nio: saturating-sub(0.0177356 div(mul(elapsed sub(0.0177356 0.0173844)) 60));\n\n/* 3. one-shot */ \n:ensure(is-zero(get(hash(order-hash() "has-executed"))) "has executed"),\n:set(hash(order-hash() "has-executed") 1);\n\n/* 4. get-elapsed */ \n_: sub(now() get(hash(order-hash() "deploy-time")));'
			);
			assert.equal(order.inputs.length, order1.inputs.length);
			assert.equal(order.outputs.length, order1.outputs.length);
			assert.equal(order.vaults.length, 3);

			assert.equal(order.vaults[0].vaultType, 'input');
			assert.equal(order.vaults[0].vaultId, BigInt(2));
			assert.equal(order.vaults[0].balance, BigInt(291));
			assert.equal(order.vaults[0].token.id, '0x0000000000000000000000000000000000000000');
			assert.equal(order.vaults[0].token.address, '0x0000000000000000000000000000000000000000');
			assert.equal(order.vaults[0].token.name, 'T2');
			assert.equal(order.vaults[0].token.symbol, 'T2');
			assert.equal(order.vaults[0].token.decimals, '0');

			assert.equal(order.vaults[1].vaultType, 'output');
			assert.equal(order.vaults[1].vaultId, BigInt(1));
			assert.equal(order.vaults[1].balance, BigInt(624419));
			assert.equal(order.vaults[1].token.id, '0x0000000000000000000000000000000000000000');
			assert.equal(order.vaults[1].token.address, '0x0000000000000000000000000000000000000000');
			assert.equal(order.vaults[1].token.name, 'T1');
			assert.equal(order.vaults[1].token.symbol, 'T1');
			assert.equal(order.vaults[1].token.decimals, '0');

			assert.equal(order.vaults[2].vaultType, 'inputOutput');
			assert.equal(order.vaults[2].vaultId, BigInt(3));
			assert.equal(order.vaults[2].balance, BigInt(30578));
			assert.equal(order.vaults[2].token.id, '0x0000000000000000000000000000000000000000');
			assert.equal(order.vaults[2].token.address, '0x0000000000000000000000000000000000000000');
			assert.equal(order.vaults[2].token.name, 'T3');
			assert.equal(order.vaults[2].token.symbol, 'T3');
			assert.equal(order.vaults[2].token.decimals, '0');
		});

		it('should measure order performance given an order id and subgraph', async () => {
			await mockServer
				.forPost('/sg1')
				.once()
				.thenReply(200, JSON.stringify({ data: { orders: [order3] } }));
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

			const raindexClient = extractWasmEncodedData(RaindexClient.new([YAML]));
			const order = extractWasmEncodedData(await raindexClient.getOrderByHash(1, 'hash'));
			const result = await order.getVaultsVolume(BigInt(1632000000), BigInt(1734571449));
			if (result.error) assert.fail('expected to resolve, but failed');

			const expected: VaultVolume[] = [
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
						netVol: '0x2b5e3af16b1880000',
						totalIn: '0x2b5e3af16b1880000',
						totalOut: '0x0',
						totalVol: '0x2b5e3af16b1880000'
					}
				},
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
						netVol: '0x56bc75e2d63100000',
						totalIn: '0x0',
						totalOut: '0x56bc75e2d63100000',
						totalVol: '0x56bc75e2d63100000'
					}
				}
			];
			assert.deepEqual(result.value, expected);
		});

		it('should measure order performance given an order id and subgraph', async () => {
			await mockServer
				.forPost('/sg1')
				.once()
				.thenReply(200, JSON.stringify({ data: { orders: [order3] } }));
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

			const raindexClient = extractWasmEncodedData(RaindexClient.new([YAML]));
			const order = extractWasmEncodedData(await raindexClient.getOrderByHash(1, 'hash'));
			const result = await order.getPerformance(BigInt(1632000000), BigInt(1734571449));
			if (result.error) assert.fail('expected to resolve, but failed');

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
			assert.deepEqual(result.value, expected);
		});
	});
});
