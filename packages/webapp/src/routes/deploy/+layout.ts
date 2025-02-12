import { REGISTRY_URL } from '$lib/constants';
import type { LayoutLoad } from './$types';

export const load: LayoutLoad = async ({ url }) => {
	// get the registry url from the url params
	const registry = url.searchParams.get('registry');
	return { registry: registry || REGISTRY_URL };
};
