import type { LayoutLoad } from './$types';
import { redirect } from '@sveltejs/kit';

export const load: LayoutLoad = async ({ params, parent }) => {
	const { strategyName } = params;
	const { registryDotrains } = await parent();

	let dotrain: string;

	try {
		const _dotrain = registryDotrains.find((dotrain) => dotrain.name === strategyName)?.dotrain;
		if (!_dotrain) {
			throw redirect(307, '/deploy');
		}
		dotrain = _dotrain;
	} catch {
		throw redirect(307, '/deploy');
	}

	return {
		dotrain,
		strategyName
	};
};
