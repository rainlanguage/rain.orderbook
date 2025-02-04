import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent, waitFor } from '@testing-library/svelte';
import ComposedRainlangModal from '../lib/components/deployment/ComposedRainlangModal.svelte';

vi.mock('svelte-codemirror-editor', async () => {
	const mockCodeMirror = (await import('../lib/__mocks__/MockComponent.svelte')).default;
	return { default: mockCodeMirror };
});

describe('ComposedRainlangModal', () => {
	let composeRainlangMock: ReturnType<typeof vi.fn>;

	beforeEach(() => {
		composeRainlangMock = vi.fn(() => Promise.resolve('mocked rainlang text'));
		vi.clearAllMocks();
	});

	it('should open modal and display rainlang text when button is clicked', async () => {
		const { getByText, getByTestId } = render(ComposedRainlangModal, {
			props: {
				composeRainlang: composeRainlangMock,
				codeMirrorStyles: {}
			}
		});

		const button = getByText('Show Rainlang');
		await fireEvent.click(button);

		await waitFor(() => {
			expect(composeRainlangMock).toHaveBeenCalled();
			expect(getByTestId('modal')).toBeInTheDocument();
		});
	});
});
