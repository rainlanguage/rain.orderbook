import { describe, it, expect, vi, beforeEach } from 'vitest';
import { resetActiveNetworkRef } from '../lib/services/resetActiveNetworkRef';
import { writable } from 'svelte/store';
import type { ConfigSource, NetworkConfigSource } from '@rainlanguage/orderbook';
import { type AppStoresInterface } from '@rainlanguage/ui-components';
describe('resetActiveNetworkRef', () => {
	let mockActiveNetworkRef: AppStoresInterface['activeNetworkRef'];
	let mockSettingsStore: AppStoresInterface['settings'];

	const createMockNetworkConfigSource = (label?: string): NetworkConfigSource => ({
		rpc: 'http://localhost:8545',
		'chain-id': 1337,
		label,
		'network-id': undefined,
		currency: undefined
	});

	beforeEach(() => {
		mockActiveNetworkRef = writable<string | undefined>(undefined);
		vi.spyOn(mockActiveNetworkRef, 'set');
	});

	it('should set activeNetworkRef to the first network key if networks exist', async () => {
		const networks: Record<string, NetworkConfigSource> = {
			network1: createMockNetworkConfigSource('Network One'),
			network2: createMockNetworkConfigSource('Network Two')
		};
		mockSettingsStore = writable<ConfigSource | undefined>({ networks } as ConfigSource);
		await resetActiveNetworkRef(mockActiveNetworkRef, mockSettingsStore);
		expect(mockActiveNetworkRef.set).toHaveBeenCalledWith('network1');
	});

	it('should set activeNetworkRef to undefined if networks object is empty', async () => {
		mockSettingsStore = writable<ConfigSource | undefined>({ networks: {} } as ConfigSource);
		await resetActiveNetworkRef(mockActiveNetworkRef, mockSettingsStore);
		expect(mockActiveNetworkRef.set).toHaveBeenCalledWith(undefined);
	});

	it('should set activeNetworkRef to undefined if networks is undefined in settings', async () => {
		mockSettingsStore = writable<ConfigSource | undefined>({ subgraphs: {} } as ConfigSource);
		await resetActiveNetworkRef(mockActiveNetworkRef, mockSettingsStore);
		expect(mockActiveNetworkRef.set).toHaveBeenCalledWith(undefined);
	});

	it('should set activeNetworkRef to undefined if settings store is undefined', async () => {
		mockSettingsStore = writable<ConfigSource | undefined>(undefined);
		await resetActiveNetworkRef(mockActiveNetworkRef, mockSettingsStore);
		expect(mockActiveNetworkRef.set).toHaveBeenCalledWith(undefined);
	});
});
