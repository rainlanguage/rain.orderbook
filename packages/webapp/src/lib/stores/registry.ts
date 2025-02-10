import { writable } from 'svelte/store';

export const registryUrl = writable<string>(
	'https://raw.githubusercontent.com/rainlanguage/rain.strategies/74096cc20b6ff6ca907d3591658b47ca279f4637/ports/registry'
);
