import { writable } from 'svelte/store';

export function cachedWritableStore<T>(
	key: string,
	defaultValue: T,
	serialize: (value: T) => string,
	deserialize: (serialized: string) => T
) {
	const getCache = () => {
		try {
			const cached = localStorage.getItem(key);
			return cached !== null ? deserialize(cached) : defaultValue;
		} catch (error) {
			return defaultValue;
		}
	};
	const setCache = (value?: T) => {
		try {
			if (value !== undefined) {
				localStorage.setItem(key, serialize(value));
			} else {
				localStorage.removeItem(key);
			}
		} catch {
			// Silently ignore localStorage errors to allow the application to function
			// without persistence in environments where localStorage is unavailable
		}
	};

	const data = writable<T>(getCache());

	data.subscribe((value) => {
		setCache(value);
	});

	return data;
}
