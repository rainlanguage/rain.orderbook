import { writable } from 'svelte/store';

export const initialPageState = {
  url: new URL('http://localhost:3000/'),
  params: {},
  form: {},
  status: 200,
  error: null,
  route: {
    id: null,
  },
};

const mockPageWritable = writable<typeof initialPageState>(initialPageState);

export const mockPageStore = {
  subscribe: mockPageWritable.subscribe,
  set: mockPageWritable.set,
  mockSetSubscribeValue: (newValue: Partial<typeof initialPageState>): void => {
    mockPageWritable.update((currentValue) => ({
      ...currentValue,
      ...newValue,
    }));
  },
  reset: () => mockPageWritable.set(initialPageState),
};
