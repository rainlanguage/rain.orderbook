import { REGISTRY_URL } from '$lib/constants';
import { validateStrategies, fetchRegistryDotrains } from '@rainlanguage/ui-components/services';
import RegistryManager from '$lib/services/RegistryManager';
import type { LayoutLoad } from './$types';

export const load: LayoutLoad = async ({ url }) => {
	const registry = url.searchParams.get('registry') || REGISTRY_URL;
	

	if (RegistryManager.isCustomRegistry(registry)) {
		RegistryManager.setToStorage(registry);
	} else {
		RegistryManager.clearFromStorage();
	}

	try {
		const registryDotrains = await fetchRegistryDotrains(registry);

		const { validStrategies, invalidStrategies } = await validateStrategies(registryDotrains);

		return {
			registry,
			registryDotrains,
			validStrategies,
			invalidStrategies,
			error: null
		};
	} catch (error: unknown) {
		return {
			registry,
			registryDotrains: [],
			validStrategies: [],
			invalidStrategies: [],
			error: error instanceof Error ? error.message : 'Unknown error occurred'
		};
	}
};
