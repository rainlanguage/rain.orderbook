import { describe, it, expect, vi, beforeEach } from 'vitest';
import { handleShareChoices } from '../lib/services/handleShareChoices';
import type { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';

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
