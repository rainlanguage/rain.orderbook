import { writable } from 'svelte/store';

const mockColorThemeWritable = writable<string>('light');

const initialPageState = {
	data: {
		stores: { settings: {} },
		dotrain: 'some dotrain content',
		deployment: { key: 'deploy-key' },
		strategyDetail: {}
	},
	url: new URL('http://localhost:3000/deploy'),
	params: {},
	form: {},
	status: 200,
	error: null,
	route: {
		id: null
	}
};

const mockPageWritable = writable<typeof initialPageState>(initialPageState);

export const mockColorThemeStore = {
  subscribe: mockColorThemeWritable.subscribe,
  set: mockColorThemeWritable.set,
  mockSetSubscribeValue: (value: string): void => mockColorThemeWritable.set(value),
};

export const mockPageStore = {
	subscribe: mockPageWritable.subscribe,
	set: mockPageWritable.set,
	mockSetSubscribeValue: (newValue: Partial<typeof initialPageState>): void => {
		mockPageWritable.update((currentValue) => ({
			...currentValue,
			...newValue
		}));
	},
	reset: () => mockPageWritable.set(initialPageState)
};
