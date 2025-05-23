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

	it('should set activeNetworkRef to the first network key if networks exist', () => {
		const networks: Record<string, NetworkConfigSource> = {
			network1: createMockNetworkConfigSource('Network One'),
			network2: createMockNetworkConfigSource('Network Two')
		};
		mockSettingsStore = writable<ConfigSource | undefined>({ networks } as ConfigSource);
		resetActiveNetworkRef(mockActiveNetworkRef, mockSettingsStore);
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		const calledValue = (mockActiveNetworkRef.set as any).mock.calls[0][0];
		expect(Object.keys(networks)).toContain(calledValue);
	});

	it('should set activeNetworkRef to undefined if networks object is empty', () => {
		mockSettingsStore = writable<ConfigSource | undefined>({ networks: {} } as ConfigSource);
		resetActiveNetworkRef(mockActiveNetworkRef, mockSettingsStore);
		expect(mockActiveNetworkRef.set).toHaveBeenCalledWith(undefined);
	});

	it('should set activeNetworkRef to undefined if networks is undefined in settings', () => {
		mockSettingsStore = writable<ConfigSource | undefined>({ subgraphs: {} } as ConfigSource);
		resetActiveNetworkRef(mockActiveNetworkRef, mockSettingsStore);
		expect(mockActiveNetworkRef.set).toHaveBeenCalledWith(undefined);
	});

	it('should set activeNetworkRef to undefined if settings store is undefined', () => {
		mockSettingsStore = writable<ConfigSource | undefined>(undefined);
		resetActiveNetworkRef(mockActiveNetworkRef, mockSettingsStore);
		expect(mockActiveNetworkRef.set).toHaveBeenCalledWith(undefined);
	});

	it('should set activeNetworkRef to undefined if settings store value is null', () => {
		mockSettingsStore = writable<ConfigSource | null | undefined>(
			null
		) as AppStoresInterface['settings'];
		resetActiveNetworkRef(mockActiveNetworkRef, mockSettingsStore);
		expect(mockActiveNetworkRef.set).toHaveBeenCalledWith(undefined);
	});

	it('should throw error if networks is null in settings', () => {
		const settingsWithNullNetworks = { networks: null } as unknown as ConfigSource;
		mockSettingsStore = writable<ConfigSource | undefined>(settingsWithNullNetworks);
		expect(() => resetActiveNetworkRef(mockActiveNetworkRef, mockSettingsStore)).toThrow(
			'Error resetting active network'
		);
		expect(mockActiveNetworkRef.set).not.toHaveBeenCalled();
	});
});
