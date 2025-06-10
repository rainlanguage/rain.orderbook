import { describe, it, expect, vi, beforeEach } from 'vitest';
import { resetActiveNetworkRef } from '../lib/services/resetActiveNetworkRef';
import { writable } from 'svelte/store';
import type { NetworkCfg, Config } from '@rainlanguage/orderbook';
import { type AppStoresInterface } from '@rainlanguage/ui-components';
describe('resetActiveNetworkRef', () => {
	let mockActiveNetworkRef: AppStoresInterface['activeNetworkRef'];
	let mockSettingsStore: AppStoresInterface['settings'];

	const createMockNetworkConfigSource = (label?: string): NetworkCfg => ({
		key: 'network1',
		rpc: 'http://localhost:8545',
		chainId: 1337,
		label,
		networkId: undefined,
		currency: undefined
	});

	beforeEach(() => {
		mockActiveNetworkRef = writable<string | undefined>(undefined);
		vi.spyOn(mockActiveNetworkRef, 'set');
	});

	it('should set activeNetworkRef to the first network key if networks exist', () => {
		const networks: Record<string, NetworkCfg> = {
			network1: createMockNetworkConfigSource('Network One'),
			network2: createMockNetworkConfigSource('Network Two')
		};
		mockSettingsStore = writable<Config>({ orderbook: { networks } } as unknown as Config);
		resetActiveNetworkRef(mockActiveNetworkRef, mockSettingsStore);
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		const calledValue = (mockActiveNetworkRef.set as any).mock.calls[0][0];
		expect(Object.keys(networks)).toContain(calledValue);
	});

	it('should set activeNetworkRef to undefined if networks object is empty', () => {
		mockSettingsStore = writable<Config>({
			orderbook: { networks: {} }
		} as unknown as Config);
		resetActiveNetworkRef(mockActiveNetworkRef, mockSettingsStore);
		expect(mockActiveNetworkRef.set).toHaveBeenCalledWith(undefined);
	});

	it('should set activeNetworkRef to undefined if networks is empty in settings', () => {
		mockSettingsStore = writable<Config>({
			orderbook: { networks: {} }
		} as unknown as Config);
		resetActiveNetworkRef(mockActiveNetworkRef, mockSettingsStore);
		expect(mockActiveNetworkRef.set).toHaveBeenCalledWith(undefined);
	});

	it('should throw error if networks is null in settings', () => {
		const settingsWithNullNetworks = { orderbook: { networks: null } } as unknown as Config;
		mockSettingsStore = writable<Config>(settingsWithNullNetworks);
		expect(() => resetActiveNetworkRef(mockActiveNetworkRef, mockSettingsStore)).toThrow(
			'Error resetting active network'
		);
		expect(mockActiveNetworkRef.set).not.toHaveBeenCalled();
	});
});
