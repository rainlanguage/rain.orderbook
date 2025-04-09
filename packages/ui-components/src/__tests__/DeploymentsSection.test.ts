import { render, screen, waitFor } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import DeploymentsSection from '../lib/components/deployment/DeploymentsSection.svelte';
import { DotrainOrderGui } from '@rainlanguage/orderbook';

describe('DeploymentsSection', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('should render deployments when promise resolves', async () => {
		// Create a promise that we can control
		const mockDeployments = new Map([
			[
				'key1',
				{ name: 'Deployment 1', description: 'Description 1', short_description: 'Short 1' }
			],
			['key2', { name: 'Deployment 2', description: 'Description 2', short_description: 'Short 2' }]
		]);
		(DotrainOrderGui.getDeploymentDetails as Mock).mockResolvedValue({ value: mockDeployments });

		render(DeploymentsSection, {
			props: {
				dotrain: 'test-dotrain',
				strategyName: 'Test Strategy'
			}
		});

		await waitFor(() => {
			expect(screen.getByText('Deployment 1')).toBeInTheDocument();
			expect(screen.getByText('Deployment 2')).toBeInTheDocument();
		});
	});

	it('should handle error when fetching deployments fails', async () => {
		const testErrorMessage = 'Test error message';
		(DotrainOrderGui.getDeploymentDetails as Mock).mockReturnValue({
			error: { msg: testErrorMessage }
		});

		render(DeploymentsSection, {
			props: {
				dotrain: 'test-dotrain',
				strategyName: 'Test Strategy'
			}
		});

		const errorMessage = await screen.findByText(testErrorMessage);
		expect(errorMessage).toBeInTheDocument();
	});

	it('should fetch deployments when dotrain prop changes', async () => {
		const { rerender } = render(DeploymentsSection, {
			props: {
				dotrain: '',
				strategyName: 'Test Strategy'
			}
		});

		expect(DotrainOrderGui.getDeploymentDetails).toHaveBeenCalledTimes(1);

		await rerender({ dotrain: 'new-dotrain', strategyName: 'Test Strategy' });

		expect(DotrainOrderGui.getDeploymentDetails).toHaveBeenCalledWith('new-dotrain');
	});
});
