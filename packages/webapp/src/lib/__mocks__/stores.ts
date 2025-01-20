import { writable } from 'svelte/store';
import { type Config } from '@wagmi/core';
import { mockWeb3Config } from './mockWeb3Config';

const mockSignerAddressWritable = writable<string>('');
const mockChainIdWritable = writable<number>(0);
const mockConnectedWritable = writable<boolean>(false);
const mockWagmiConfigWritable = writable<Config>(mockWeb3Config);


export const mockSignerAddressStore = {
	subscribe: mockSignerAddressWritable.subscribe,
	set: mockSignerAddressWritable.set,
	mockSetSubscribeValue: (value: string): void => mockSignerAddressWritable.set(value)
};

export const mockChainIdStore = {
	subscribe: mockChainIdWritable.subscribe,
	set: mockChainIdWritable.set,
	mockSetSubscribeValue: (value: number): void => mockChainIdWritable.set(value)
};

export const mockConnectedStore = {
	subscribe: mockConnectedWritable.subscribe,
	set: mockConnectedWritable.set,
	mockSetSubscribeValue: (value: boolean): void => mockConnectedWritable.set(value)
};

export const mockWagmiConfigStore = {
	subscribe: mockWagmiConfigWritable.subscribe,
	set: mockWagmiConfigWritable.set,
	mockSetSubscribeValue: (value: Config): void => mockWagmiConfigWritable.set(value)
};
