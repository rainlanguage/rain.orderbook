import { REGISTRY_URL } from '$lib/constants';
import { fetchRegistryDotrains } from '@rainlanguage/ui-components/services';
import type { LayoutLoad } from './$types';
import { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';

export const load: LayoutLoad = async ({ url }) => {
	// get the registry url from the url params
	const registry = url.searchParams.get('registry');

	const registryDotrains = await fetchRegistryDotrains(registry || REGISTRY_URL);
	const strategyDetails = await Promise.all(
		registryDotrains.map(async (registryDotrain) => {
			const result = await DotrainOrderGui.getStrategyDetails(registryDotrain.dotrain);
			if (result.error) {
				throw new Error(result.error.msg);
			}
			return { ...registryDotrain, details: result.value };
		})
	);

	return { registry: registry || REGISTRY_URL, registryDotrains, strategyDetails };
};
