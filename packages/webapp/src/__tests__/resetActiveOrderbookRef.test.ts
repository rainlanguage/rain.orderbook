import { describe, it, expect, vi, beforeEach } from 'vitest';
import { resetActiveOrderbookRef } from '../lib/services/resetActiveOrderbookRef';
import { writable } from 'svelte/store';
import type { OrderbookCfg } from '@rainlanguage/orderbook';
import { type AppStoresInterface } from '@rainlanguage/ui-components';

const createMockOrderbookConfig = (address: string, label?: string): OrderbookCfg => ({
	address,
	label,
	key: address,
	network: {
		key: 'mainnet',
		chainId: 1,
		rpc: 'https://mainnet.infura.io/v3/your-infura-project-id'
	},
	subgraph: {
		key: 'mainnet',
		url: 'https://api.subgraph.com/subgraphs/name/your-subgraph-name/your-subgraph-id'
	}
});

describe('resetActiveOrderbookRef', () => {
	let mockActiveOrderbookRef: AppStoresInterface['activeOrderbookRef'];
	let mockActiveNetworkOrderbooksStore: AppStoresInterface['activeNetworkOrderbooks'];

	beforeEach(() => {
		mockActiveOrderbookRef = writable<string | undefined>(undefined);
		vi.spyOn(mockActiveOrderbookRef, 'set');
	});

	it('should set activeOrderbookRef to the first orderbook key if orderbooks exist', () => {
		const orderbooks: Record<string, OrderbookCfg> = {
			orderbook1: createMockOrderbookConfig('0x123', 'Orderbook One'),
			orderbook2: createMockOrderbookConfig('0x456', 'Orderbook Two')
		};
		mockActiveNetworkOrderbooksStore = writable<Record<string, OrderbookCfg>>(orderbooks);
		resetActiveOrderbookRef(mockActiveOrderbookRef, mockActiveNetworkOrderbooksStore);
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		const calledValue = (mockActiveOrderbookRef.set as any).mock.calls[0][0];
		expect(Object.keys(orderbooks)).toContain(calledValue);
	});

	it('should set activeOrderbookRef to undefined if orderbooks object is empty', () => {
		mockActiveNetworkOrderbooksStore = writable<Record<string, OrderbookCfg>>({});
		resetActiveOrderbookRef(mockActiveOrderbookRef, mockActiveNetworkOrderbooksStore);
		expect(mockActiveOrderbookRef.set).toHaveBeenCalledWith(undefined);
	});

	it('should set activeOrderbookRef to undefined if activeNetworkOrderbooksStore is undefined', () => {
		mockActiveNetworkOrderbooksStore = writable<Record<string, OrderbookCfg>>(undefined);
		resetActiveOrderbookRef(mockActiveOrderbookRef, mockActiveNetworkOrderbooksStore);
		expect(mockActiveOrderbookRef.set).toHaveBeenCalledWith(undefined);
	});

	it('should throw error if activeNetworkOrderbooksStore value is null', () => {
		mockActiveNetworkOrderbooksStore = writable<Record<string, OrderbookCfg> | null>(
			null
		) as AppStoresInterface['activeNetworkOrderbooks'];
		expect(() =>
			resetActiveOrderbookRef(mockActiveOrderbookRef, mockActiveNetworkOrderbooksStore)
		).toThrow('Error resetting active orderbook');
		expect(mockActiveOrderbookRef.set).not.toHaveBeenCalled();
	});
});
