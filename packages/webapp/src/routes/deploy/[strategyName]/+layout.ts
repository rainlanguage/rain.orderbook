import type { LayoutLoad } from './$types';
import { redirect } from '@sveltejs/kit';
import type { NameAndDescriptionCfg } from '@rainlanguage/orderbook/js_api';
export const load: LayoutLoad = async ({ params, parent }) => {
	const { strategyName } = params;
	const { registryDotrains, validStrategies } = await parent();

	let dotrain: string;
	let strategyDetail: NameAndDescriptionCfg;

	try {
		const _dotrain = registryDotrains.find((dotrain) => dotrain.name === strategyName)?.dotrain;
		if (!_dotrain) {
			throw redirect(307, '/deploy');
		}
		dotrain = _dotrain;
		const _strategyDetail = validStrategies.find((detail) => detail.name === strategyName)?.details;
		if (!_strategyDetail) {
			throw redirect(307, '/deploy');
		}
		strategyDetail = _strategyDetail;
	} catch {
		throw redirect(307, '/deploy');
	}

	return {
		dotrain,
		strategyName,
		strategyDetail,
		pageName: strategyName
	};
};
