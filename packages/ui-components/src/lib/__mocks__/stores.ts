import type { ConfigSource } from '$lib/typeshare/config';
import { writable } from 'svelte/store';
import settingsFixture from '../__fixtures__/settings-12-11-24.json';

const mockSettingsWritable = writable<ConfigSource | undefined>(settingsFixture);
const mockActiveSubgraphsWritable = writable<Record<string, string>>({});
const mockAccountsWritable = writable<Record<string, string>>({});
const mockActiveAccountsItemsWritable = writable<Record<string, string>>({});
const mockActiveOrderStatusWritable = writable<boolean | undefined>(undefined);
const mockOrderHashWritable = writable<string>('');
const mockHideZeroBalanceVaultsWritable = writable<boolean>(false);

export const mockSettingsStore = {
	subscribe: mockSettingsWritable.subscribe,
	set: mockSettingsWritable.set,
	mockSetSubscribeValue: (value: ConfigSource | undefined): void => mockSettingsWritable.set(value)
};

export const mockActiveSubgraphsStore = {
	subscribe: mockActiveSubgraphsWritable.subscribe,
	set: mockActiveSubgraphsWritable.set,
	mockSetSubscribeValue: (value: Record<string, string>): void =>
		mockActiveSubgraphsWritable.set(value)
};

export const mockAccountsStore = {
	subscribe: mockAccountsWritable.subscribe
};

export const mockActiveAccountsItemsStore = {
	subscribe: mockActiveAccountsItemsWritable.subscribe,
	set: mockActiveAccountsItemsWritable.set,
	mockSetSubscribeValue: (value: Record<string, string>): void =>
		mockActiveAccountsItemsWritable.set(value)
};

export const mockActiveOrderStatusStore = {
	subscribe: mockActiveOrderStatusWritable.subscribe,
	set: mockActiveOrderStatusWritable.set,
	mockSetSubscribeValue: (value: boolean | undefined): void =>
		mockActiveOrderStatusWritable.set(value)
};

export const mockOrderHashStore = {
	subscribe: mockOrderHashWritable.subscribe,
	set: mockOrderHashWritable.set,
	mockSetSubscribeValue: (value: string): void => mockOrderHashWritable.set(value)
};

export const mockHideZeroBalanceVaultsStore = {
	subscribe: mockHideZeroBalanceVaultsWritable.subscribe,
	set: mockHideZeroBalanceVaultsWritable.set,
	mockSetSubscribeValue: (value: boolean): void => mockHideZeroBalanceVaultsWritable.set(value)
};
