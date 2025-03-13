import { REGISTRY_URL } from '$lib/constants';
import { fetchRegistryDotrains } from '@rainlanguage/ui-components/services';
import type { LayoutLoad } from './$types';
import { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';
import type { ValidStrategyDetail, InvalidStrategyDetail } from '@rainlanguage/ui-components';

export const load: LayoutLoad = async ({ url }) => {
	const registry = url.searchParams.get('registry');
	try {
		const registryDotrains = await fetchRegistryDotrains(registry || REGISTRY_URL);

		const validStrategies: ValidStrategyDetail[] = [];
		const invalidStrategies: InvalidStrategyDetail[] = [];

		await Promise.all(
			registryDotrains.map(async (registryDotrain) => {
				try {
					const details = await DotrainOrderGui.getStrategyDetails(registryDotrain.dotrain);
					validStrategies.push({ ...registryDotrain, details });
				} catch (error) {
					invalidStrategies.push({
						name: registryDotrain.name,
						error: error as string
					});
				}
			})
		);

		return {
			registry: registry || REGISTRY_URL,
			registryDotrains,
			validStrategies,
			invalidStrategies,
			error: null
		};
	} catch (error: unknown) {
		return {
			registry: registry || REGISTRY_URL,
			registryDotrains: [],
			validStrategies: [],
			invalidStrategies: [],
			error: error instanceof Error ? error.message : 'Unknown error occurred'
		};
	}
};
