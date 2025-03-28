import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import { handleShareChoices } from '../lib/services/handleShareChoices';
import { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';

describe('handleShareChoices', () => {
	let guiInstance: DotrainOrderGui;

	beforeEach(() => {
		guiInstance = new DotrainOrderGui();

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
		(DotrainOrderGui.prototype.serializeState as Mock).mockReturnValue({ value: 'mockState123' });

		await handleShareChoices(guiInstance);

		expect(navigator.clipboard.writeText).toHaveBeenCalledWith(
			'http://example.com/?state=mockState123'
		);
	});

	it('should handle null state', async () => {
		(DotrainOrderGui.prototype.serializeState as Mock).mockReturnValue({ value: null });

		await handleShareChoices(guiInstance);

		expect(navigator.clipboard.writeText).toHaveBeenCalledWith('http://example.com/?state=');
	});
});
