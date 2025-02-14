import type { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';
import { page } from '$app/stores';
import { get } from 'svelte/store';

export async function handleShareChoices(gui: DotrainOrderGui) {
	// get the current url
	const url = get(page).url;
	// get the current state
	const state = gui?.serializeState();
	url.searchParams.set('state', state || '');
	navigator.clipboard.writeText(url.toString());
}

// tests
if (import.meta.vitest) {
	const { describe, it, expect, vi } = import.meta.vitest;

	describe('handleShareChoices', () => {
		beforeEach(() => {
			// Mock clipboard API
			Object.assign(navigator, {
				clipboard: {
					writeText: vi.fn()
				}
			});

			// Mock Svelte's page store
			vi.mock('$app/stores', () => ({
				page: {
					subscribe: vi.fn((fn) => {
						fn({ url: new URL('http://example.com') });
						return () => {};
					})
				}
			}));
		});

		it('should share the choices with state', async () => {
			const mockGui = {
				serializeState: vi.fn().mockReturnValue('mockState123')
			};

			await handleShareChoices(mockGui as unknown as DotrainOrderGui);

			expect(navigator.clipboard.writeText).toHaveBeenCalledWith(
				'http://example.com/?state=mockState123'
			);
		});

		it('should handle null state', async () => {
			const mockGui = {
				serializeState: vi.fn().mockReturnValue(null)
			};

			await handleShareChoices(mockGui as unknown as DotrainOrderGui);

			expect(navigator.clipboard.writeText).toHaveBeenCalledWith('http://example.com/?state=');
		});

		it('should handle undefined gui', async () => {
			await handleShareChoices(undefined as unknown as DotrainOrderGui);

			expect(navigator.clipboard.writeText).toHaveBeenCalledWith('http://example.com/?state=');
		});
	});
}
