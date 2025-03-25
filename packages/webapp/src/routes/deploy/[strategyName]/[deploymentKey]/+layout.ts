import { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';
import type { LayoutLoad } from '../$types';

interface LayoutParentData {
	dotrain: string;
}

export const load: LayoutLoad = async ({ params, parent }) => {
	const { deploymentKey } = params;
	const { dotrain } = (await parent()) as unknown as LayoutParentData;

	const result = await DotrainOrderGui.getDeploymentDetail(dotrain, deploymentKey || '');
	if (result.error) {
		throw new Error(result.error.msg);
	}
	const { name, description } = result.value;

	return {
		deployment: { key: deploymentKey, name, description },
		dotrain,
		pageName: deploymentKey
	};
};
