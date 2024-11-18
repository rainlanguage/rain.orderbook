import { writable } from 'svelte/store';

const mockWalletAddressMatchesOrBlankWritable = writable<() => boolean>(() => false);

export const mockWalletAddressMatchesOrBlankStore = {
  subscribe: mockWalletAddressMatchesOrBlankWritable.subscribe,
  set: mockWalletAddressMatchesOrBlankWritable.set,
  mockSetSubscribeValue: (value: () => boolean): void =>
    mockWalletAddressMatchesOrBlankWritable.set(value),
};
