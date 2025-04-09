
import RegistryManager from '$lib/services/RegistryManager';
import { fetchRegistryDotrains } from '@rainlanguage/ui-components/services';
import type { Mock } from 'vitest';

export async function loadRegistryUrl(url: string): Promise<void> {
	if (!url) {
		throw new Error('No URL provided');
	}

	try {
		await fetchRegistryDotrains(url);
		RegistryManager.setToStorage(url);
		RegistryManager.updateUrlParam(url);
		window.location.reload();
	} catch (e) {
		const errorMessage = e instanceof Error ? e.message : 'Failed to update registry URL';
		throw new Error(errorMessage);
	}
}



if (import.meta.vitest) {
	const { describe, it, expect, vi, beforeEach } = import.meta.vitest;

	// Mock dependencies
	vi.mock('$lib/services/RegistryManager', () => ({
		default: {
			setToStorage: vi.fn(),
			updateUrlParam: vi.fn()
		}
	}));

	vi.mock('@rainlanguage/ui-components/services', () => ({
		fetchRegistryDotrains: vi.fn()
	}));

	describe('loadRegistryUrl', () => {
		beforeEach(() => {
			vi.resetAllMocks();
			// Reset window.location
			delete (global.window as any).location;
			(global.window as any).location = { reload: vi.fn() };
		});

		it('should throw an error if no URL is provided', async () => {
			await expect(loadRegistryUrl('')).rejects.toThrow('No URL provided');
		});

		it('should successfully load registry URL and reload the page', async () => {
			const testUrl = 'https://example.com/registry';
			
			(fetchRegistryDotrains as Mock).mockResolvedValueOnce(undefined);

			await loadRegistryUrl(testUrl);

			// Verify fetchRegistryDotrains was called with the correct URL
			expect(fetchRegistryDotrains).toHaveBeenCalledWith(testUrl);

			// Verify RegistryManager methods were called
			expect(RegistryManager.setToStorage).toHaveBeenCalledWith(testUrl);
			expect(RegistryManager.updateUrlParam).toHaveBeenCalledWith(testUrl);

			// Verify page reload
			expect(window.location.reload).toHaveBeenCalled();
		});

		it('should throw an error if fetching registry dotrains fails', async () => {
			const testUrl = 'https://example.com/registry';
			const errorMessage = 'Fetch failed';

			(fetchRegistryDotrains as Mock).mockRejectedValueOnce(new Error(errorMessage));

			await expect(loadRegistryUrl(testUrl)).rejects.toThrow(errorMessage);

			// Verify RegistryManager and reload were not called
			expect(RegistryManager.setToStorage).not.toHaveBeenCalled();
			expect(RegistryManager.updateUrlParam).not.toHaveBeenCalled();
			expect(window.location.reload).not.toHaveBeenCalled();
		});

		it('should handle non-Error exception during registry fetch', async () => {
			const testUrl = 'https://example.com/registry';

			(fetchRegistryDotrains as Mock).mockRejectedValueOnce('String error');

			await expect(loadRegistryUrl(testUrl)).rejects.toThrow('Failed to update registry URL');

			// Verify RegistryManager and reload were not called
			expect(RegistryManager.setToStorage).not.toHaveBeenCalled();
			expect(RegistryManager.updateUrlParam).not.toHaveBeenCalled();
			expect(window.location.reload).not.toHaveBeenCalled();
		});
	});
}

