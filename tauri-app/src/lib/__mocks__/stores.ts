import { writable } from 'svelte/store';

const mockColorThemeWritable = writable<string>('light');

export const mockColorThemeStore = {
  subscribe: mockColorThemeWritable.subscribe,
  set: mockColorThemeWritable.set,
  mockSetSubscribeValue: (value: string): void => mockColorThemeWritable.set(value),
};
