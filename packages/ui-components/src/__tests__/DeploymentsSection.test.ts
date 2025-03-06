import { render, screen, waitFor } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import DeploymentsSection from '../lib/components/deployment/DeploymentsSection.svelte';
import { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';

// Mock the DotrainOrderGui
vi.mock('@rainlanguage/orderbook/js_api', () => ({
	DotrainOrderGui: {
		getDeploymentDetails: vi.fn()
	}
}));

describe('DeploymentsSection', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	  it('should render deployments when promise resolves', async () => {
    // Create a promise that we can control
    const deploymentPromise = Promise.resolve(new Map([
      [
        'key1',
        { name: 'Deployment 1', description: 'Description 1', short_description: 'Short 1' }
      ],
      ['key2', { name: 'Deployment 2', description: 'Description 2', short_description: 'Short 2' }]
    ]));
    
    vi.mocked(DotrainOrderGui.getDeploymentDetails).mockReturnValue(deploymentPromise);

    render(DeploymentsSection, {
      props: {
        dotrain: 'test-dotrain',
        strategyName: 'Test Strategy'
      }
    });

    // Wait for the promise to resolve and the component to update
    await deploymentPromise;
    
    // Check that the deployments are rendered
    await waitFor(() => {
      expect(screen.getByText('Deployment 1')).toBeInTheDocument();
      expect(screen.getByText('Deployment 2')).toBeInTheDocument();
    });
  });

	it('should handle error when fetching deployments fails', async () => {
		const testErrorMessage = "Test error message";
		vi.mocked(DotrainOrderGui.getDeploymentDetails).mockRejectedValue(new Error(testErrorMessage));

		render(DeploymentsSection, {
			props: {
				dotrain: 'test-dotrain',
				strategyName: 'Test Strategy'
			}
		});

		const errorMessage = await screen.findByText(
			testErrorMessage
		);
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
