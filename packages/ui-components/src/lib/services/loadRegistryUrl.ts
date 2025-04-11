import { RegistryManager } from '../providers/registry/RegistryManager';
import { fetchRegistryDotrains } from './registry';
import type { Mock } from 'vitest';
import { initialRegistry } from '../__mocks__/stores';

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

if (import.meta.vitest) {
	const { describe, it, expect, vi, beforeEach } = import.meta.vitest;

	// Mock dependencies
	vi.mock('./registry', () => ({
		fetchRegistryDotrains: vi.fn()
	}));

	describe('loadRegistryUrl', () => {
		beforeEach(() => {
			vi.resetAllMocks();
			// Reset window.location
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			(global.window as any).location = undefined;
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			(global.window as any).location = { reload: vi.fn() };
		});

		it('should throw an error if no URL is provided', async () => {
			const mockRegistryManager = initialRegistry as RegistryManager;
			await expect(loadRegistryUrl('', mockRegistryManager)).rejects.toThrow('No URL provided');
		});

		it('should throw an error if no registry manager is provided', async () => {
			await expect(
				loadRegistryUrl('https://example.com/registry', null as unknown as RegistryManager)
			).rejects.toThrow('Registry manager is required');
		});

		it('should successfully load registry URL and reload the page', async () => {
			const testUrl = 'https://example.com/registry';
			const mockRegistryManager = initialRegistry as RegistryManager;

			(fetchRegistryDotrains as Mock).mockResolvedValueOnce(undefined);
			await loadRegistryUrl(testUrl, mockRegistryManager);
			expect(fetchRegistryDotrains).toHaveBeenCalledWith(testUrl);
			expect(mockRegistryManager.setRegistry).toHaveBeenCalledWith(testUrl);
			expect(window.location.reload).toHaveBeenCalled();
		});

		it('should throw an error if fetching registry dotrains fails', async () => {
			const testUrl = 'https://example.com/registry';
			const errorMessage = 'Fetch failed';
			const mockRegistryManager = {
				setRegistry: vi.fn()
			} as unknown as RegistryManager;

			(fetchRegistryDotrains as Mock).mockRejectedValueOnce(new Error(errorMessage));

			await expect(loadRegistryUrl(testUrl, mockRegistryManager)).rejects.toThrow(errorMessage);

			expect(mockRegistryManager.setRegistry).not.toHaveBeenCalled();
			expect(window.location.reload).not.toHaveBeenCalled();
		});

		it('should handle non-Error exception during registry fetch', async () => {
			const testUrl = 'https://example.com/registry';
			const mockRegistryManager = {
				setRegistry: vi.fn()
			} as unknown as RegistryManager;

			(fetchRegistryDotrains as Mock).mockRejectedValueOnce('String error');

			await expect(loadRegistryUrl(testUrl, mockRegistryManager)).rejects.toThrow(
				'Failed to update registry URL'
			);

			expect(mockRegistryManager.setRegistry).not.toHaveBeenCalled();
			expect(window.location.reload).not.toHaveBeenCalled();
		});
	});
}
