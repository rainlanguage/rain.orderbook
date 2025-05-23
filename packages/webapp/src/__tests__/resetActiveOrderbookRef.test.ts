import { describe, it, expect, vi, beforeEach } from 'vitest';
import { resetActiveOrderbookRef } from '../lib/services/resetActiveOrderbookRef';
import { writable } from 'svelte/store';
import type { OrderbookConfigSource } from '@rainlanguage/orderbook';
import { type AppStoresInterface } from '@rainlanguage/ui-components';

const createMockOrderbookConfigSource = (
	address: string,
	label?: string
): OrderbookConfigSource => ({
	address,
	label,
	network: undefined,
	subgraph: undefined
});

describe('resetActiveOrderbookRef', () => {
	let mockActiveOrderbookRef: AppStoresInterface['activeOrderbookRef'];
	let mockActiveNetworkOrderbooksStore: AppStoresInterface['activeNetworkOrderbooks'];

	beforeEach(() => {
		mockActiveOrderbookRef = writable<string | undefined>(undefined);
		vi.spyOn(mockActiveOrderbookRef, 'set');
	});

	it('should set activeOrderbookRef to the first orderbook key if orderbooks exist', () => {
		const orderbooks: Record<string, OrderbookConfigSource> = {
			orderbook1: createMockOrderbookConfigSource('0x123', 'Orderbook One'),
			orderbook2: createMockOrderbookConfigSource('0x456', 'Orderbook Two')
		};
		mockActiveNetworkOrderbooksStore = writable<Record<string, OrderbookConfigSource>>(orderbooks);
		resetActiveOrderbookRef(mockActiveOrderbookRef, mockActiveNetworkOrderbooksStore);
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		const calledValue = (mockActiveOrderbookRef.set as any).mock.calls[0][0];
		expect(Object.keys(orderbooks)).toContain(calledValue);
	});

	it('should set activeOrderbookRef to undefined if orderbooks object is empty', () => {
		mockActiveNetworkOrderbooksStore = writable<Record<string, OrderbookConfigSource>>({});
		resetActiveOrderbookRef(mockActiveOrderbookRef, mockActiveNetworkOrderbooksStore);
		expect(mockActiveOrderbookRef.set).toHaveBeenCalledWith(undefined);
	});

	it('should set activeOrderbookRef to undefined if activeNetworkOrderbooksStore is undefined', () => {
		mockActiveNetworkOrderbooksStore = writable<Record<string, OrderbookConfigSource>>(undefined);
		resetActiveOrderbookRef(mockActiveOrderbookRef, mockActiveNetworkOrderbooksStore);
		expect(mockActiveOrderbookRef.set).toHaveBeenCalledWith(undefined);
	});

	it('should throw error if activeNetworkOrderbooksStore value is null', () => {
		mockActiveNetworkOrderbooksStore = writable<Record<string, OrderbookConfigSource> | null>(
			null
		) as AppStoresInterface['activeNetworkOrderbooks'];
		expect(() =>
			resetActiveOrderbookRef(mockActiveOrderbookRef, mockActiveNetworkOrderbooksStore)
		).toThrow('Error resetting active orderbook');
		expect(mockActiveOrderbookRef.set).not.toHaveBeenCalled();
	});
});
