import { registryUrl } from '$lib/stores/registry';
import type { LayoutLoad } from './$types';

export const load: LayoutLoad = async ({ url }) => {
	// get the registry url from the url params
	const registry = url.searchParams.get('registry');
	if (registry) {
		registryUrl.set(registry);
	}
	return { registry };
};
