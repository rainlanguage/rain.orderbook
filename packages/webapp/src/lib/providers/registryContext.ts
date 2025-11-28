import { getContext, setContext } from 'svelte';
import type { DotrainRegistry } from '@rainlanguage/orderbook';

const REGISTRY_CONTEXT_KEY = 'dotrain-registry';

export type RegistryContext = {
	registry: DotrainRegistry | null;
	registryUrl: string;
};

export const setRegistryContext = (context: RegistryContext) => {
	setContext(REGISTRY_CONTEXT_KEY, context);
};

export const getRegistryContext = (): RegistryContext => {
	const ctx = getContext<RegistryContext>(REGISTRY_CONTEXT_KEY);
	if (!ctx) {
		throw new Error('Registry context not found. Ensure it is set in the root layout.');
	}
	return ctx;
};
