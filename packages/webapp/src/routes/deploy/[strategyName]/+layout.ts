import type { LayoutLoad } from './$types';
import { redirect } from '@sveltejs/kit';

export const load: LayoutLoad = async ({ params, parent }) => {
	const { strategyName } = params;
	const { registryDotrains } = await parent();

	try {
		const dotrain = registryDotrains.find((dotrain) => dotrain.name === strategyName)?.dotrain;
		return {
			dotrain,
			strategyName
		};
	} catch {
		throw redirect(307, '/deploy');
	}
};
