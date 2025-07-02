import assert from 'assert';
import { afterAll, beforeAll, beforeEach, describe, it } from 'vitest';
import {
	WasmEncodedResult,
	RaindexClient,
	SgOrder,
	SgTrade,
	OrderPerformance,
	VaultVolume,
	SgVault
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

		const order3 = {
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
			assert.equal(orders[0].id, order1.id);
			assert.equal(orders[1].id, order2.id);

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

	describe('Vaults', async function () {
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

		it('should get vaults', async function () {
			await mockServer
				.forPost('/sg1')
				.thenReply(200, JSON.stringify({ data: { vaults: [vault1] } }));
			await mockServer
				.forPost('/sg2')
				.thenReply(200, JSON.stringify({ data: { vaults: [vault2] } }));

			const raindexClient = extractWasmEncodedData(RaindexClient.new([YAML]));
			const result = extractWasmEncodedData(
				await raindexClient.getVaults(
					undefined,
					{
						owners: [],
						hideZeroBalance: false
					},
					1
				)
			);

			assert.equal(result.length, 2);
			assert.equal(result[0].vaultId, BigInt(vault1.vaultId));
			assert.equal(result[0].owner, vault1.owner);
			assert.equal(result[0].balance, BigInt(vault1.balance));
			assert.equal(result[0].token.id, vault1.token.id);
			assert.equal(result[0].token.address, vault1.token.address);
			assert.equal(result[0].token.name, vault1.token.name);
			assert.equal(result[0].token.symbol, vault1.token.symbol);
			assert.equal(result[0].token.decimals, BigInt(vault1.token.decimals ?? 0));
			assert.equal(result[1].vaultId, BigInt(vault2.vaultId));
			assert.equal(result[1].owner, vault2.owner);
			assert.equal(result[1].balance, BigInt(vault2.balance));
			assert.equal(result[1].token.id, vault2.token.id);
			assert.equal(result[1].token.address, vault2.token.address);
			assert.equal(result[1].token.name, vault2.token.name);
			assert.equal(result[1].token.symbol, vault2.token.symbol);
			assert.equal(result[1].token.decimals, BigInt(vault2.token.decimals ?? 0));
		});

		it('should get vault', async function () {
			await mockServer.forPost('/sg1').thenReply(200, JSON.stringify({ data: { vault: vault1 } }));

			const raindexClient = extractWasmEncodedData(RaindexClient.new([YAML]));
			const result = extractWasmEncodedData(await raindexClient.getVault(1, 'vault1'));

			assert.equal(result.vaultId, BigInt(vault1.vaultId));
			assert.equal(result.owner, vault1.owner);
			assert.equal(result.balance, BigInt(vault1.balance));
			assert.equal(result.token.id, vault1.token.id);
			assert.equal(result.token.address, vault1.token.address);
			assert.equal(result.token.name, vault1.token.name);
			assert.equal(result.token.symbol, vault1.token.symbol);
			assert.equal(result.token.decimals, BigInt(vault1.token.decimals ?? 0));
			assert.equal(result.orderbook, vault1.orderbook.id);
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
				.forPost('/sg1')
				.once()
				.thenReply(200, JSON.stringify({ data: { vault: vault1 } }));
			await mockServer
				.forPost('/sg1')
				.thenReply(200, JSON.stringify({ data: { vaultBalanceChanges: mockVaultBalanceChanges } }));

			const raindexClient = extractWasmEncodedData(RaindexClient.new([YAML]));
			const vault = extractWasmEncodedData(await raindexClient.getVault(1, 'vault1'));
			const result = extractWasmEncodedData(await vault.getBalanceChanges());

			assert.equal(result[0].type, 'deposit');
			assert.equal(result[0].amount, BigInt('5000000000000000000'));
			assert.equal(result[0].newVaultBalance, BigInt('5000000000000000000'));
			assert.equal(result[0].oldVaultBalance, BigInt('0'));
			assert.equal(result[0].timestamp, BigInt('1734054063'));
			assert.equal(
				result[0].transaction.id,
				'0x85857b5c6d0b277f9e971b6b45cab98720f90b8f24d65df020776d675b71fc22'
			);
			assert.equal(result[0].transaction.from, '0x7177b9d00bB5dbcaaF069CC63190902763783b09');
			assert.equal(result[0].transaction.blockNumber, BigInt('34407047'));
			assert.equal(result[0].transaction.timestamp, BigInt('1734054063'));
			assert.equal(result[0].orderbook, '0xCEe8Cd002F151A536394E564b84076c41bBBcD4d');
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
			await mockServer
				.forPost('/sg1')
				.once()
				.thenReply(200, JSON.stringify({ data: { vault: vault1 } }));
			await mockServer
				.forPost('/sg1')
				.once()
				.thenReply(200, JSON.stringify({ data: { order } }));

			const raindexClient = extractWasmEncodedData(RaindexClient.new([YAML]));
			const vault = extractWasmEncodedData(await raindexClient.getVault(1, 'vault1'));
			const res = extractWasmEncodedData(await vault.getDepositCalldata('500'));
			assert.equal(res.length, 330);
		});

		it('should handle zero deposit amount', async () => {
			await mockServer
				.forPost('/sg1')
				.once()
				.thenReply(200, JSON.stringify({ data: { vault: vault1 } }));
			await mockServer.forPost('/sg1').thenReply(200, JSON.stringify({ data: { order } }));

			const raindexClient = extractWasmEncodedData(RaindexClient.new([YAML]));
			const vault = extractWasmEncodedData(await raindexClient.getVault(1, 'vault1'));

			const res = await vault.getDepositCalldata('0');
			if (!res.error) assert.fail('expected to reject, but resolved');
			assert.equal(res.error.msg, 'Zero amount');
		});

		it('should throw error for invalid deposit amount', async () => {
			await mockServer
				.forPost('/sg1')
				.once()
				.thenReply(200, JSON.stringify({ data: { vault: vault1 } }));
			const raindexClient = extractWasmEncodedData(RaindexClient.new([YAML]));
			const vault = extractWasmEncodedData(await raindexClient.getVault(1, 'vault1'));

			const res = await vault.getDepositCalldata('-100');
			if (!res.error) assert.fail('expected to reject, but resolved');
			assert.equal(res.error.msg, 'invalid digit: -');
		});

		it('should get withdraw calldata for a vault', async () => {
			await mockServer
				.forPost('/sg1')
				.once()
				.thenReply(200, JSON.stringify({ data: { vault: vault1 } }));
			await mockServer.forPost('/sg1').thenReply(200, JSON.stringify({ data: { order } }));

			const raindexClient = extractWasmEncodedData(RaindexClient.new([YAML]));
			const vault = extractWasmEncodedData(await raindexClient.getVault(1, 'vault1'));

			let res = await vault.getWithdrawCalldata('500');
			if (res.error) assert.fail('expected to resolve, but failed');
			assert.equal(res.value.length, 330);

			res = await vault.getWithdrawCalldata('0');
			if (!res.error) assert.fail('expected to reject, but resolved');
			assert.equal(res.error.msg, 'Zero amount');
			assert.equal(res.error.readableMsg, 'Amount cannot be zero');
		});

		it('should read allowance for a vault', async () => {
			await mockServer
				.forPost('/sg1')
				.once()
				.thenReply(200, JSON.stringify({ data: { vault: vault1 } }));
			await mockServer.forPost('/rpc1').thenReply(
				200,
				JSON.stringify({
					jsonrpc: '2.0',
					id: 1,
					result: '0x0000000000000000000000000000000000000000000000000000000000000064'
				})
			);

			const raindexClient = extractWasmEncodedData(RaindexClient.new([YAML]));
			const vault = extractWasmEncodedData(await raindexClient.getVault(1, 'vault1'));
			const res = extractWasmEncodedData(await vault.getAllowance());
			assert.equal(res, '0x64');
		});

		it('should generate valid approval calldata with correct length', async () => {
			await mockServer
				.forPost('/sg1')
				.once()
				.thenReply(200, JSON.stringify({ data: { vault: vault1 } }));
			await mockServer.forPost('/rpc1').thenReply(
				200,
				JSON.stringify({
					jsonrpc: '2.0',
					id: 1,
					result: '0x0000000000000000000000000000000000000000000000000000000000000064'
				})
			);

			const raindexClient = extractWasmEncodedData(RaindexClient.new([YAML]));
			const vault = extractWasmEncodedData(await raindexClient.getVault(1, 'vault1'));

			const res = extractWasmEncodedData(await vault.getApprovalCalldata('600'));
			assert.ok(res.startsWith('0x'));
			assert.equal(res.length, 138);
		});

		it('should handle approval amount equal to allowance', async () => {
			await mockServer
				.forPost('/sg1')
				.once()
				.thenReply(200, JSON.stringify({ data: { vault: vault1 } }));
			// Allowance is 100, and user tries to approve 100, so there should be no approval calldata
			await mockServer.forPost('/rpc1').thenReply(
				200,
				JSON.stringify({
					jsonrpc: '2.0',
					id: 1,
					result: '0x0000000000000000000000000000000000000000000000000000000000000064'
				})
			);

			const raindexClient = extractWasmEncodedData(RaindexClient.new([YAML]));
			const vault = extractWasmEncodedData(await raindexClient.getVault(1, 'vault1'));

			const res = await vault.getApprovalCalldata('100');
			if (!res.error) assert.fail('expected to reject, but resolved');
			assert.equal(res.error.msg, 'Existing allowance');
		});

		it('should handle approval amount less than allowance', async () => {
			await mockServer
				.forPost('/sg1')
				.once()
				.thenReply(200, JSON.stringify({ data: { vault: vault1 } }));
			// Allowance is 100, and user tries to approve 90, so there should be approval calldata
			await mockServer.forPost('/rpc1').thenReply(
				200,
				JSON.stringify({
					jsonrpc: '2.0',
					id: 1,
					result: '0x0000000000000000000000000000000000000000000000000000000000000064'
				})
			);

			const raindexClient = extractWasmEncodedData(RaindexClient.new([YAML]));
			const vault = extractWasmEncodedData(await raindexClient.getVault(1, 'vault1'));

			const res = await vault.getApprovalCalldata('90');
			if (!res.error) assert.fail('expected to reject, but resolved');
			assert.equal(res.error.msg, 'Existing allowance');
		});
	});
});
