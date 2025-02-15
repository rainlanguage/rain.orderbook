import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import DisclaimerModal from '../lib/components/deployment/DisclaimerModal.svelte';

describe('DisclaimerModal', () => {
	const mockOnAccept = vi.fn();

	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('calls onAccept when accepting disclaimer', async () => {
		render(DisclaimerModal, {
			props: {
				open: true,
				onAccept: mockOnAccept
			}
		});

		const deployButton = await screen.findByText('Deploy');
		await fireEvent.click(deployButton);

		expect(mockOnAccept).toHaveBeenCalled();
	});
});
