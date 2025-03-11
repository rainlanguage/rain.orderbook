import { render, screen } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import DeploymentsSection from '../lib/components/deployment/DeploymentsSection.svelte';
import {
	DotrainOrderGui,
	type DeploymentDetails,
	type WasmEncodedResult
} from '@rainlanguage/orderbook/js_api';

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

	it('should render deployments when data is available', async () => {
		const mockDeployments = new Map([
			[
				'key1',
				{ name: 'Deployment 1', description: 'Description 1', short_description: 'Short 1' }
			],
			['key2', { name: 'Deployment 2', description: 'Description 2', short_description: 'Short 2' }]
		]) as unknown as WasmEncodedResult<DeploymentDetails>;

		vi.mocked(DotrainOrderGui.getDeploymentDetails).mockResolvedValue(mockDeployments);

		render(DeploymentsSection, {
			props: {
				dotrain: 'test-dotrain',
				strategyName: 'Test Strategy'
			}
		});

		// Wait for deployments to load
		const deployment1 = await screen.findByText('Deployment 1');
		const deployment2 = await screen.findByText('Deployment 2');

		expect(deployment1).toBeInTheDocument();
		expect(deployment2).toBeInTheDocument();
	});

	it('should handle error when fetching deployments fails', async () => {
		vi.mocked(DotrainOrderGui.getDeploymentDetails).mockRejectedValue(new Error('API Error'));

		render(DeploymentsSection, {
			props: {
				dotrain: 'test-dotrain',
				strategyName: 'Test Strategy'
			}
		});

		const errorMessage = await screen.findByText(
			'Error loading deployments: Error getting deployments.'
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

		expect(DotrainOrderGui.getDeploymentDetails).not.toHaveBeenCalled();

		await rerender({ dotrain: 'new-dotrain', strategyName: 'Test Strategy' });

		expect(DotrainOrderGui.getDeploymentDetails).toHaveBeenCalledWith('new-dotrain');
	});
});
