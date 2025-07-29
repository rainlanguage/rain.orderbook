import type { LayoutLoad } from './$types';
import { redirect } from '@sveltejs/kit';
import type { NameAndDescriptionCfg } from '@rainlanguage/orderbook';
export const load: LayoutLoad = async ({ params, parent }) => {
	const { orderName } = params;
	const { registryDotrains, validOrders } = await parent();

	let dotrain: string;
	let orderDetail: NameAndDescriptionCfg;

	try {
		const _dotrain = registryDotrains.find((dotrain) => dotrain.name === orderName)?.dotrain;
		if (!_dotrain) {
			throw redirect(307, '/deploy');
		}
		dotrain = _dotrain;
		const _orderDetail = validOrders.find((detail) => detail.name === orderName)?.details;
		if (!_orderDetail) {
			throw redirect(307, '/deploy');
		}
		orderDetail = _orderDetail;
	} catch {
		throw redirect(307, '/deploy');
	}

	return {
		dotrain,
		orderName,
		orderDetail,
		pageName: orderName
	};
};
