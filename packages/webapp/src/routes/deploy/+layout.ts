import { REGISTRY_URL } from '$lib/constants';
import type { LayoutLoad } from './$types';
import type { InvalidStrategyDetail, ValidStrategyDetail } from '@rainlanguage/ui-components';
import { fetchRegistryDotrains, validateStrategies } from '@rainlanguage/ui-components/services';
import type { RegistryDotrain } from '@rainlanguage/ui-components/services';

/**
+ * Type defining the structure of the load function's return value,
+ * including registry information and validation results.
+ */
type LoadResult = {
	registryFromUrl: string;
	registryDotrains: RegistryDotrain[];
	validStrategies: ValidStrategyDetail[];
	invalidStrategies: InvalidStrategyDetail[];
	error: string | null;
};

export const load: LayoutLoad<LoadResult> = async ({ url }) => {
	const registryFromUrl = url.searchParams.get('registry') || REGISTRY_URL;

	try {
		const registryDotrains = await fetchRegistryDotrains(registryFromUrl);

		const { validStrategies, invalidStrategies } = await validateStrategies(registryDotrains);

		return {
			registryFromUrl,
			registryDotrains,
			validStrategies,
			invalidStrategies,
			error: null
		};
	} catch (error: unknown) {
		return {
			registryFromUrl,
			registryDotrains: [],
			validStrategies: [],
			invalidStrategies: [],
			error: error instanceof Error ? error.message : 'Unknown error occurred'
		};
	}
};
