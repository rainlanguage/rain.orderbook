import { describe, it, expect, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import type { NetworkSyncStatus, OrderbookSyncStatus } from '@rainlanguage/orderbook';
import {
	networkStatuses,
	orderbookStatuses,
	updateNetworkStatus,
	updateOrderbookStatus,
	updateStatus,
	aggregateStatus
} from '../lib/stores/localDbStatus';

describe('localDbStatus store', () => {
	beforeEach(() => {
		networkStatuses.set(new Map());
		orderbookStatuses.set(new Map());
	});

	describe('networkStatuses', () => {
		it('starts with empty map', () => {
			const map = get(networkStatuses);
			expect(map.size).toBe(0);
		});

		it('can set network statuses directly', () => {
			const status: NetworkSyncStatus = {
				chainId: 137,
				status: 'active',
				schedulerState: 'leader'
			};
			networkStatuses.update((map) => {
				map.set(137, status);
				return new Map(map);
			});

			const map = get(networkStatuses);
			expect(map.size).toBe(1);
			expect(map.get(137)).toEqual(status);
		});
	});

	describe('orderbookStatuses', () => {
		it('starts with empty map', () => {
			const map = get(orderbookStatuses);
			expect(map.size).toBe(0);
		});
	});

	describe('updateNetworkStatus', () => {
		it('adds new network status to map', () => {
			const status: NetworkSyncStatus = {
				chainId: 137,
				status: 'syncing',
				schedulerState: 'leader'
			};

			updateNetworkStatus(status);

			const map = get(networkStatuses);
			expect(map.size).toBe(1);
			expect(map.get(137)).toEqual(status);
		});

		it('updates existing network status', () => {
			const initial: NetworkSyncStatus = {
				chainId: 137,
				status: 'syncing',
				schedulerState: 'leader'
			};
			const updated: NetworkSyncStatus = {
				chainId: 137,
				status: 'active',
				schedulerState: 'leader'
			};

			updateNetworkStatus(initial);
			updateNetworkStatus(updated);

			const map = get(networkStatuses);
			expect(map.size).toBe(1);
			expect(map.get(137)?.status).toBe('active');
		});

		it('handles multiple networks independently', () => {
			const polygon: NetworkSyncStatus = {
				chainId: 137,
				status: 'active',
				schedulerState: 'leader'
			};
			const arbitrum: NetworkSyncStatus = {
				chainId: 42161,
				status: 'syncing',
				schedulerState: 'leader'
			};

			updateNetworkStatus(polygon);
			updateNetworkStatus(arbitrum);

			const map = get(networkStatuses);
			expect(map.size).toBe(2);
			expect(map.get(137)?.status).toBe('active');
			expect(map.get(42161)?.status).toBe('syncing');
		});

		it('stores error field when present', () => {
			const status: NetworkSyncStatus = {
				chainId: 137,
				status: 'failure',
				schedulerState: 'leader',
				error: 'RPC connection failed'
			};

			updateNetworkStatus(status);

			const map = get(networkStatuses);
			expect(map.get(137)?.error).toBe('RPC connection failed');
		});
	});

	describe('updateOrderbookStatus', () => {
		it('adds new orderbook status to map', () => {
			const status: OrderbookSyncStatus = {
				obId: {
					chainId: 137,
					orderbookAddress: '0x1234567890123456789012345678901234567890'
				},
				status: 'syncing',
				schedulerState: 'leader',
				phaseMessage: 'Fetching latest block'
			};

			updateOrderbookStatus(status);

			const map = get(orderbookStatuses);
			expect(map.size).toBe(1);
			const key = '137:0x1234567890123456789012345678901234567890';
			expect(map.get(key)).toEqual(status);
		});

		it('updates existing orderbook status', () => {
			const initial: OrderbookSyncStatus = {
				obId: {
					chainId: 137,
					orderbookAddress: '0x1234567890123456789012345678901234567890'
				},
				status: 'syncing',
				schedulerState: 'leader',
				phaseMessage: 'Fetching latest block'
			};
			const updated: OrderbookSyncStatus = {
				obId: {
					chainId: 137,
					orderbookAddress: '0x1234567890123456789012345678901234567890'
				},
				status: 'active',
				schedulerState: 'leader'
			};

			updateOrderbookStatus(initial);
			updateOrderbookStatus(updated);

			const map = get(orderbookStatuses);
			expect(map.size).toBe(1);
			const key = '137:0x1234567890123456789012345678901234567890';
			expect(map.get(key)?.status).toBe('active');
		});

		it('handles multiple orderbooks on same chain', () => {
			const ob1: OrderbookSyncStatus = {
				obId: {
					chainId: 137,
					orderbookAddress: '0x1111111111111111111111111111111111111111'
				},
				status: 'active',
				schedulerState: 'leader'
			};
			const ob2: OrderbookSyncStatus = {
				obId: {
					chainId: 137,
					orderbookAddress: '0x2222222222222222222222222222222222222222'
				},
				status: 'syncing',
				schedulerState: 'leader'
			};

			updateOrderbookStatus(ob1);
			updateOrderbookStatus(ob2);

			const map = get(orderbookStatuses);
			expect(map.size).toBe(2);
		});

		it('handles orderbooks on different chains', () => {
			const polygonOb: OrderbookSyncStatus = {
				obId: {
					chainId: 137,
					orderbookAddress: '0x1234567890123456789012345678901234567890'
				},
				status: 'active',
				schedulerState: 'leader'
			};
			const arbitrumOb: OrderbookSyncStatus = {
				obId: {
					chainId: 42161,
					orderbookAddress: '0x1234567890123456789012345678901234567890'
				},
				status: 'syncing',
				schedulerState: 'leader'
			};

			updateOrderbookStatus(polygonOb);
			updateOrderbookStatus(arbitrumOb);

			const map = get(orderbookStatuses);
			expect(map.size).toBe(2);
		});

		it('stores error field when present', () => {
			const status: OrderbookSyncStatus = {
				obId: {
					chainId: 137,
					orderbookAddress: '0x1234567890123456789012345678901234567890'
				},
				status: 'failure',
				schedulerState: 'leader',
				error: 'Decode error'
			};

			updateOrderbookStatus(status);

			const map = get(orderbookStatuses);
			const key = '137:0x1234567890123456789012345678901234567890';
			expect(map.get(key)?.error).toBe('Decode error');
		});
	});

	describe('updateStatus', () => {
		it('dispatches NetworkSyncStatus to updateNetworkStatus', () => {
			const status: NetworkSyncStatus = {
				chainId: 137,
				status: 'active',
				schedulerState: 'leader'
			};

			updateStatus(status);

			const networkMap = get(networkStatuses);
			const orderbookMap = get(orderbookStatuses);
			expect(networkMap.size).toBe(1);
			expect(orderbookMap.size).toBe(0);
			expect(networkMap.get(137)).toEqual(status);
		});

		it('dispatches OrderbookSyncStatus to updateOrderbookStatus', () => {
			const status: OrderbookSyncStatus = {
				obId: {
					chainId: 137,
					orderbookAddress: '0x1234567890123456789012345678901234567890'
				},
				status: 'syncing',
				schedulerState: 'leader'
			};

			updateStatus(status);

			const networkMap = get(networkStatuses);
			const orderbookMap = get(orderbookStatuses);
			expect(networkMap.size).toBe(0);
			expect(orderbookMap.size).toBe(1);
		});

		it('distinguishes types by presence of obId field', () => {
			const networkStatus: NetworkSyncStatus = {
				chainId: 137,
				status: 'active',
				schedulerState: 'leader'
			};
			const orderbookStatus: OrderbookSyncStatus = {
				obId: {
					chainId: 42161,
					orderbookAddress: '0x1234567890123456789012345678901234567890'
				},
				status: 'syncing',
				schedulerState: 'leader'
			};

			updateStatus(networkStatus);
			updateStatus(orderbookStatus);

			const networkMap = get(networkStatuses);
			const orderbookMap = get(orderbookStatuses);
			expect(networkMap.size).toBe(1);
			expect(orderbookMap.size).toBe(1);
		});
	});

	describe('callback integration', () => {
		it('routes mixed status updates to correct stores via single updateStatus function', () => {
			const networkStatus1: NetworkSyncStatus = {
				chainId: 137,
				status: 'active',
				schedulerState: 'leader'
			};
			const orderbookStatus1: OrderbookSyncStatus = {
				obId: {
					chainId: 137,
					orderbookAddress: '0x1234567890123456789012345678901234567890'
				},
				status: 'syncing',
				schedulerState: 'leader',
				phaseMessage: 'Fetching latest block'
			};
			const networkStatus2: NetworkSyncStatus = {
				chainId: 42161,
				status: 'syncing',
				schedulerState: 'leader'
			};
			const orderbookStatus2: OrderbookSyncStatus = {
				obId: {
					chainId: 42161,
					orderbookAddress: '0xabcdefabcdefabcdefabcdefabcdefabcdefabcd'
				},
				status: 'active',
				schedulerState: 'leader'
			};

			updateStatus(networkStatus1);
			updateStatus(orderbookStatus1);
			updateStatus(networkStatus2);
			updateStatus(orderbookStatus2);

			const networkMap = get(networkStatuses);
			const orderbookMap = get(orderbookStatuses);

			expect(networkMap.size).toBe(2);
			expect(orderbookMap.size).toBe(2);

			expect(networkMap.get(137)?.status).toBe('active');
			expect(networkMap.get(42161)?.status).toBe('syncing');

			const obKey1 = '137:0x1234567890123456789012345678901234567890';
			const obKey2 = '42161:0xabcdefabcdefabcdefabcdefabcdefabcdefabcd';
			expect(orderbookMap.get(obKey1)?.status).toBe('syncing');
			expect(orderbookMap.get(obKey1)?.phaseMessage).toBe('Fetching latest block');
			expect(orderbookMap.get(obKey2)?.status).toBe('active');
		});

		it('simulates scheduler callback receiving interleaved network and orderbook updates', () => {
			const updates = [
				{
					chainId: 137,
					status: 'syncing' as const,
					schedulerState: 'leader' as const
				},
				{
					obId: {
						chainId: 137,
						orderbookAddress: '0x1111111111111111111111111111111111111111'
					},
					status: 'syncing' as const,
					schedulerState: 'leader' as const,
					phaseMessage: 'Running bootstrap'
				},
				{
					obId: {
						chainId: 137,
						orderbookAddress: '0x1111111111111111111111111111111111111111'
					},
					status: 'active' as const,
					schedulerState: 'leader' as const
				},
				{
					chainId: 137,
					status: 'active' as const,
					schedulerState: 'leader' as const
				}
			];

			for (const update of updates) {
				updateStatus(update as NetworkSyncStatus | OrderbookSyncStatus);
			}

			const networkMap = get(networkStatuses);
			const orderbookMap = get(orderbookStatuses);

			expect(networkMap.get(137)?.status).toBe('active');
			const obKey = '137:0x1111111111111111111111111111111111111111';
			expect(orderbookMap.get(obKey)?.status).toBe('active');
		});
	});

	describe('aggregateStatus', () => {
		it('returns active when both maps are empty', () => {
			const result = get(aggregateStatus);
			expect(result.status).toBe('active');
			expect(result.error).toBeUndefined();
		});

		it('returns active when all statuses are active', () => {
			updateNetworkStatus({
				chainId: 137,
				status: 'active',
				schedulerState: 'leader'
			});
			updateOrderbookStatus({
				obId: {
					chainId: 137,
					orderbookAddress: '0x1234567890123456789012345678901234567890'
				},
				status: 'active',
				schedulerState: 'leader'
			});

			const result = get(aggregateStatus);
			expect(result.status).toBe('active');
			expect(result.error).toBeUndefined();
		});

		it('returns syncing when any status is syncing and none are failure', () => {
			updateNetworkStatus({
				chainId: 137,
				status: 'active',
				schedulerState: 'leader'
			});
			updateNetworkStatus({
				chainId: 42161,
				status: 'syncing',
				schedulerState: 'leader'
			});

			const result = get(aggregateStatus);
			expect(result.status).toBe('syncing');
			expect(result.error).toBeUndefined();
		});

		it('returns failure when any network has failure', () => {
			updateNetworkStatus({
				chainId: 137,
				status: 'active',
				schedulerState: 'leader'
			});
			updateNetworkStatus({
				chainId: 42161,
				status: 'failure',
				schedulerState: 'leader',
				error: 'RPC timeout'
			});

			const result = get(aggregateStatus);
			expect(result.status).toBe('failure');
			expect(result.error).toBe('RPC timeout');
		});

		it('returns failure when any orderbook has failure', () => {
			updateNetworkStatus({
				chainId: 137,
				status: 'active',
				schedulerState: 'leader'
			});
			updateOrderbookStatus({
				obId: {
					chainId: 137,
					orderbookAddress: '0x1234567890123456789012345678901234567890'
				},
				status: 'failure',
				schedulerState: 'leader',
				error: 'Decode error'
			});

			const result = get(aggregateStatus);
			expect(result.status).toBe('failure');
			expect(result.error).toBe('Decode error');
		});

		it('failure takes precedence over syncing', () => {
			updateNetworkStatus({
				chainId: 137,
				status: 'syncing',
				schedulerState: 'leader'
			});
			updateNetworkStatus({
				chainId: 42161,
				status: 'failure',
				schedulerState: 'leader',
				error: 'Connection refused'
			});

			const result = get(aggregateStatus);
			expect(result.status).toBe('failure');
			expect(result.error).toBe('Connection refused');
		});

		it('syncing takes precedence over active', () => {
			updateNetworkStatus({
				chainId: 137,
				status: 'active',
				schedulerState: 'leader'
			});
			updateOrderbookStatus({
				obId: {
					chainId: 137,
					orderbookAddress: '0x1234567890123456789012345678901234567890'
				},
				status: 'syncing',
				schedulerState: 'leader',
				phaseMessage: 'Building SQL batch'
			});

			const result = get(aggregateStatus);
			expect(result.status).toBe('syncing');
		});

		it('returns first failure error when multiple failures exist', () => {
			updateNetworkStatus({
				chainId: 137,
				status: 'failure',
				schedulerState: 'leader',
				error: 'First error'
			});
			updateNetworkStatus({
				chainId: 42161,
				status: 'failure',
				schedulerState: 'leader',
				error: 'Second error'
			});

			const result = get(aggregateStatus);
			expect(result.status).toBe('failure');
			expect(result.error).toBeDefined();
		});

		it('updates reactively when statuses change', () => {
			updateNetworkStatus({
				chainId: 137,
				status: 'active',
				schedulerState: 'leader'
			});

			let result = get(aggregateStatus);
			expect(result.status).toBe('active');

			updateNetworkStatus({
				chainId: 137,
				status: 'syncing',
				schedulerState: 'leader'
			});

			result = get(aggregateStatus);
			expect(result.status).toBe('syncing');

			updateNetworkStatus({
				chainId: 137,
				status: 'failure',
				schedulerState: 'leader',
				error: 'Error occurred'
			});

			result = get(aggregateStatus);
			expect(result.status).toBe('failure');
		});
	});
});
