import type { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';
import { page } from '$app/stores';
import { get } from 'svelte/store';

export async function handleShareChoices(gui: DotrainOrderGui, registryUrl: string) {
	// get the current url
	const url = get(page).url;

	// get the current state
	const result = gui.serializeState();
	if (result.error) {
		throw new Error(result.error.msg);
	}
	const state = result.value;
	url.searchParams.set('state', state || '');
	url.searchParams.set('registry', registryUrl);
	navigator.clipboard.writeText(url.toString());
}
