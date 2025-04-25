import { writable } from 'svelte/store';

export function cachedWritableStore<T>(
	key: string,
	defaultValue: T,
	serialize: (value: T) => string,
	deserialize: (serialized: string) => T
) {
	const getCache = () => {
		const cached = localStorage.getItem(key);
		return cached !== null ? deserialize(cached) : defaultValue;
	};
	const setCache = (value?: T) => {
		if (value !== undefined) {
			localStorage.setItem(key, serialize(value));
		} else {
			localStorage.removeItem(key);
		}
	};

	const data = writable<T>(getCache());

	data.subscribe((value) => {
		setCache(value);
	});

	return data;
}