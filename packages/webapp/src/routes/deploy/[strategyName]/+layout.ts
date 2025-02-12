import { rawDotrain } from '$lib/stores/raw-dotrain';
import { get } from 'svelte/store';
import type { LayoutLoad } from './$types';
import { redirect } from '@sveltejs/kit';
import { getFileRegistry } from '$lib/services/getFileRegistry';

export const load: LayoutLoad = async ({ fetch, params, parent }) => {
	const { strategyName } = params;
	const { registry } = await parent();

	let dotrain;

	try {
		if (strategyName === 'raw' && get(rawDotrain)) {
			dotrain = get(rawDotrain);
		} else {
			const fileList = await getFileRegistry(registry);

			const strategy = fileList.find((file: { name: string }) => file.name === strategyName);
			if (!strategy) {
				throw new Error(`Strategy ${strategyName} not found`);
			}

			const dotrainResponse = await fetch(strategy.url);
			dotrain = await dotrainResponse.text();
		}
	} catch {
		throw redirect(307, '/deploy');
	}

	return {
		dotrain,
		strategyName
	};
};
