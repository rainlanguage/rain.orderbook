import { REGISTRY_URL } from '$lib/constants';
import { fetchRegistryDotrains } from '@rainlanguage/ui-components/services';
import type { LayoutLoad } from './$types';
import { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';
import type { StrategyDetail } from '@rainlanguage/ui-components';

export const load: LayoutLoad = async ({ url }) => {
	const registry = url.searchParams.get('registry');
	try {
		const registryDotrains = await fetchRegistryDotrains(registry || REGISTRY_URL);
		const strategyDetails: StrategyDetail[] = await Promise.all(
			registryDotrains.map(async (registryDotrain) => {
				try {
					const details = await DotrainOrderGui.getStrategyDetails(registryDotrain.dotrain);
					return { ...registryDotrain, details };
				} catch (error) {
					return { ...registryDotrain, details: null, error };
				}
			})
		);
		return {
			registry: registry || REGISTRY_URL,
			registryDotrains,
			strategyDetails,
			error: null
		};
	} catch (error: unknown) {
		return {
			registry: registry || REGISTRY_URL,
			registryDotrains: [],
			strategyDetails: [],
			error: error instanceof Error ? error.message : 'Unknown error occurred'
		};
	}
};
