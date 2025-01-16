import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import DeploymentSteps from '../lib/components/deployment/DeploymentSteps.svelte';
import { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';
import dotrain from '../lib/__fixtures__/dotrain_for_testing.rain?raw';
import userEvent from '@testing-library/user-event';

vi.mock('@rainlanguage/orderbook/js_api', () => ({
	DotrainOrderGui: {
		getAvailableDeployments: vi.fn(),
		chooseDeployment: vi.fn()
	}
}));

describe('DeploymentSteps', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('renders strategy URL input and load button initially', () => {
		render(DeploymentSteps);
		expect(screen.getByPlaceholderText('Enter URL to .rain file')).toBeInTheDocument();
		const loadButton = screen.getByText('Load Strategy');
		expect(loadButton).toBeInTheDocument();
		expect(loadButton).toBeDisabled();
	});

	it('enables load button when URL is entered', async () => {
		render(DeploymentSteps);
		const urlInput = screen.getByPlaceholderText('Enter URL to .rain file');
		const loadButton = screen.getByText('Load Strategy');

		await userEvent.type(urlInput, 'https://example.com/strategy.rain');
		await waitFor(() => {
			expect(loadButton).not.toBeDisabled();
		});
	});

	it('loads strategy from URL when button is clicked', async () => {
		global.fetch = vi.fn().mockResolvedValue({
			ok: true,
			text: () => Promise.resolve(JSON.stringify(dotrain))
		});

		render(DeploymentSteps);
		const urlInput = screen.getByPlaceholderText('Enter URL to .rain file');
		const loadButton = screen.getByText('Load Strategy');
		await userEvent.clear(urlInput);
		await fireEvent.input(urlInput, { target: { value: 'https://example.com/strategy.rain' } });
		await userEvent.click(loadButton);

		await waitFor(() => {
			expect(global.fetch).toHaveBeenCalledWith('https://example.com/strategy.rain');
		});
	});

	it('shows deployments dropdown after strategy is loaded', async () => {
		global.fetch = vi.fn().mockResolvedValue({
			ok: true,
			text: () => Promise.resolve(JSON.stringify(dotrain))
		});

		const mockDeployments = [
			{ key: 'deployment1', label: 'Deployment 1' },
			{ key: 'deployment2', label: 'Deployment 2' }
		];

		(DotrainOrderGui.getAvailableDeployments as Mock).mockResolvedValue(mockDeployments);

		render(DeploymentSteps);
		const urlInput = screen.getByPlaceholderText('Enter URL to .rain file');
		const loadButton = screen.getByText('Load Strategy');

		await userEvent.type(urlInput, 'https://example.com/strategy.rain');
		await userEvent.click(loadButton);

		await waitFor(() => {
			expect(screen.getByText('Deployments')).toBeInTheDocument();
			expect(screen.getByText('Select a deployment')).toBeInTheDocument();
		});
	});

	it('handles URL fetch errors', async () => {
		global.fetch = vi.fn().mockRejectedValue(new Error('Failed to fetch'));

		render(DeploymentSteps);
		const urlInput = screen.getByPlaceholderText('Enter URL to .rain file');
		const loadButton = screen.getByText('Load Strategy');

		await userEvent.type(urlInput, 'https://example.com/strategy.rain');
		await userEvent.click(loadButton);

		await waitFor(() => {
			expect(screen.getByText('No valid strategy exists at this URL')).toBeInTheDocument();
		});
	});

	it('initializes GUI when deployment is selected', async () => {
		global.fetch = vi.fn().mockResolvedValue({
			ok: true,
			text: () => Promise.resolve(JSON.stringify(dotrain))
		});

		const mockDeployments = [
			{ key: 'deployment1', label: 'Deployment 1' },
			{ key: 'deployment2', label: 'Deployment 2' }
		];

		(DotrainOrderGui.getAvailableDeployments as Mock).mockResolvedValue(mockDeployments);

		render(DeploymentSteps);
		const urlInput = screen.getByPlaceholderText('Enter URL to .rain file');
		const loadButton = screen.getByText('Load Strategy');

		await userEvent.type(urlInput, 'https://example.com/strategy.rain');
		await userEvent.click(loadButton);

		await waitFor(() => {
			expect(screen.getByText('Deployments')).toBeInTheDocument();
			expect(screen.getByText('Select a deployment')).toBeInTheDocument();
		});

		const dropdownButton = screen.getByTestId('dropdown-button');
		await userEvent.click(dropdownButton);
		const dropdown = screen.getByTestId('dropdown');
		await userEvent.click(dropdown);
		const deploymentOption = screen.getByText('deployment1');
		await userEvent.click(deploymentOption);

		await waitFor(() => {
			expect(DotrainOrderGui.chooseDeployment).toHaveBeenCalled();
		});
	});
});
