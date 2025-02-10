import { writable } from 'svelte/store';

export const registryUrl = writable<string>(
	'https://raw.githubusercontent.com/rainlanguage/rain.strategies/ebde1801fe4a1cb5e6ce76778d5f7852cf1af634/ports/registry'
);
