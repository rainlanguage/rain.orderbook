import { RegistryManager } from '../providers/registry/RegistryManager';
import { fetchRegistryDotrains } from './registry';

export async function loadRegistryUrl(
	url: string,
	registryManager: RegistryManager
): Promise<void> {
	if (!url) {
		throw new Error('No URL provided');
	}

	if (!registryManager) {
		throw new Error('Registry manager is required');
	}

	try {
		await fetchRegistryDotrains(url);
		registryManager.setRegistry(url);
		window.location.reload();
	} catch (e) {
		const errorMessage = e instanceof Error ? e.message : 'Failed to update registry URL';
		throw new Error(errorMessage);
	}
}
