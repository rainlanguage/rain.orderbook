import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent, waitFor } from '@testing-library/svelte';
import ComposedRainlangModal from '../lib/components/deployment/ComposedRainlangModal.svelte';
import type { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';

vi.mock('svelte-codemirror-editor', async () => {
	const mockCodeMirror = (await import('../lib/__mocks__/MockComponent.svelte')).default;
	return { default: mockCodeMirror };
});

const mockGui = {
	getComposedRainlang: vi.fn(() => Promise.resolve('mocked rainlang text'))
} as unknown as DotrainOrderGui;

describe('ComposedRainlangModal', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('should open modal and display rainlang text when button is clicked', async () => {
		const { getByText, getByTestId } = render(ComposedRainlangModal, {
			props: {
				gui: mockGui
			}
		});

		const button = getByText('Show Rainlang');
		await fireEvent.click(button);

		await waitFor(() => {
			expect(mockGui.getComposedRainlang).toHaveBeenCalled();
			expect(getByTestId('modal')).toBeInTheDocument();
		});
	});
});
