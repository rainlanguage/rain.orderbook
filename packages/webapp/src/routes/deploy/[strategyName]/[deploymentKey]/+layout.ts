import { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';
import type { LayoutLoad } from '../$types';
import type { ExtendedTokenInfo } from '@rainlanguage/ui-components';

interface LayoutParentData {
	dotrain: string;
}

export const load: LayoutLoad = async ({ params, parent }) => {
	const { deploymentKey } = params;
	const { dotrain } = (await parent()) as unknown as LayoutParentData;
	const getTokenList = async () => {
		const response = await fetch('https://ipfs.io/ipns/tokens.uniswap.org');
		return await response.json();
	};

	const tokenListData = await getTokenList();
	const tokenList: ExtendedTokenInfo[] = tokenListData.tokens || [];

	const { name, description } = await DotrainOrderGui.getDeploymentDetail(
		dotrain,
		deploymentKey || ''
	);

	return { deployment: { key: deploymentKey, name, description }, dotrain, tokenList };
};
