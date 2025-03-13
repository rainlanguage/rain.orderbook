import { REGISTRY_URL } from '$lib/constants';
import { fetchRegistryDotrains } from '@rainlanguage/ui-components/services';
import type { LayoutLoad } from './$types';
import { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';

export const load: LayoutLoad = async ({ url }) => {
	const registry = url.searchParams.get('registry');
	try {
		const registryDotrains = await fetchRegistryDotrains(registry || REGISTRY_URL);
		const strategyDetails = await Promise.all(
			registryDotrains.map(async (registryDotrain) => {
				try {
					const result = await DotrainOrderGui.getStrategyDetails(registryDotrain.dotrain);
					if (result.error) {
						throw new Error(result.error.msg);
					}
					return { ...registryDotrain, details: result.value };
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
