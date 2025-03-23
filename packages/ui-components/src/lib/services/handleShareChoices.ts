import type { DotrainOrderGui } from '@rainlanguage/orderbook';
import { page } from '$app/stores';
import { get } from 'svelte/store';

export async function handleShareChoices(gui: DotrainOrderGui) {
	// get the current url
	const url = get(page).url;
	// get the current state
	const state = gui?.serializeState();
	url.searchParams.set('state', state || '');
	navigator.clipboard.writeText(url.toString());
}
