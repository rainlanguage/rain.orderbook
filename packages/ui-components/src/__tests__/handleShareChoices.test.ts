import { describe, it, expect, vi, beforeEach } from 'vitest';
import { handleShareChoices } from '../lib/services/handleShareChoices';
import type { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';
import fc from 'fast-check';

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

describe('property-based tests', () => {
	beforeEach(() => {
		Object.assign(navigator, {
			clipboard: {
				writeText: vi.fn()
			}
		});

		vi.mock('$app/stores', () => ({
			page: {
				subscribe: vi.fn((fn) => {
					fn({ url: new URL('http://example.com') });
					return () => {};
				})
			}
		}));
	});

	it('should always create valid URLs with any state string', async () => {
		await fc.assert(
			fc.asyncProperty(fc.string(), async (state) => {
				const mockGui = {
					serializeState: vi.fn().mockReturnValue(state)
				};

				await handleShareChoices(mockGui as unknown as DotrainOrderGui);

				const clipboardText = (navigator.clipboard.writeText as any).mock.calls[0][0];
				const url = new URL(clipboardText);
				
				// Property: URL should always be valid and contain the state parameter
				expect(url.searchParams.has('state')).toBe(true);
				// Compare with the encoded state value
				expect(url.searchParams.get('state')).toBe(encodeURIComponent(state));
			})
		);
	});
});
