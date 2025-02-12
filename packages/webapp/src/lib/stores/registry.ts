import { writable } from 'svelte/store';

export const registryUrl = writable<string>(
	'https://raw.githubusercontent.com/rainlanguage/rain.strategies/3b4ef719fc60064d62fff1366afd97d5715ddd4a/ports/registry'
);
