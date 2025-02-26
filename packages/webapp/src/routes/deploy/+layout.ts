import { REGISTRY_URL } from '$lib/constants';
import { fetchRegistryDotrains } from '@rainlanguage/ui-components/services';
import type { LayoutLoad } from './$types';
import { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';

export const load: LayoutLoad = async ({ url }) => {
	const registry = url.searchParams.get('registry');
	try {
		const registryDotrains = await fetchRegistryDotrains(registry || REGISTRY_URL);
		if (!registryDotrains || registryDotrains.length === 0) {
			throw new Error('No strategy registry found at URL');
		}
		const strategyDetails = await Promise.all(
			registryDotrains.map(async (registryDotrain) => {
				try {	
					const details = await DotrainOrderGui.getStrategyDetails(registryDotrain.dotrain);
					if (!details) {
						throw new Error('This registry contains invalid Dotrain documents.');
					}
					return { ...registryDotrain, details };
				} catch (error) {
					throw new Error(error instanceof Error ? error.message : String(error));
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
