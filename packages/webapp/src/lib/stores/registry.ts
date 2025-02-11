import { writable } from 'svelte/store';

export const registryUrl = writable<string>(
	'https://raw.githubusercontent.com/rainlanguage/rain.strategies/5e52d73c0a231df25bc131dfd118ff52a5824662/ports/registry'
);
