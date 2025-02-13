import { rawDotrain } from '$lib/stores/raw-dotrain';
import { get } from 'svelte/store';
import type { LayoutLoad } from './$types';
import { redirect } from '@sveltejs/kit';

export const load: LayoutLoad = async ({ params, parent }) => {
	const { strategyName } = params;
	const { registryDotrains } = await parent();

	let dotrain;

	try {
		if (strategyName === 'raw' && get(rawDotrain)) {
			dotrain = get(rawDotrain);
		} else {
			dotrain = registryDotrains.find((dotrain) => dotrain.name === strategyName)?.dotrain;
		}
	} catch {
		throw redirect(307, '/deploy');
	}

	return {
		dotrain,
		strategyName
	};
};
