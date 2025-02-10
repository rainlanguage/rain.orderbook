import { writable } from 'svelte/store';

export const registryUrl = writable<string>(
	'https://raw.githubusercontent.com/rainlanguage/rain.strategies/921222a38b480421b93d16f2fddaeb1416cb94e9/ports/registry'
);
