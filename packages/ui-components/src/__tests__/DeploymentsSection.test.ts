import { render, screen, waitFor } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import { readable, writable, type Writable } from 'svelte/store';

const mockUseRegistry = vi.fn();

vi.mock('$lib/providers/registry/useRegistry', () => ({
	useRegistry: () => mockUseRegistry()
}));

import DeploymentsSection from '../lib/components/deployment/DeploymentsSection.svelte';

describe('DeploymentsSection', () => {
	type DeploymentResult = {
		value?: Map<
			string,
			{
				name: string;
				description: string;
				short_description?: string;
			}
		>;
		error?: { msg?: string; readableMsg?: string };
	};

	type MockRegistry = {
		getDeploymentDetails: Mock<[string], DeploymentResult>;
	};

	let registryStore: Writable<MockRegistry | null>;

	beforeEach(() => {
		vi.clearAllMocks();
		registryStore = writable<MockRegistry | null>(null);

		mockUseRegistry.mockReturnValue({
			registry: registryStore,
			loading: writable(false),
			error: writable<string | null>(null),
			setRegistryUrl: vi.fn(),
			registryUrl: writable(''),
			isCustomRegistry: readable(false),
			appendRegistryToHref: vi.fn((href: string) => href)
		});
	});

	const setRegistry = (registry: MockRegistry) => {
		registryStore.set(registry);
	};

	it('should render deployments when promise resolves', async () => {
		const mockDeployments = new Map<string, { name: string; description: string }>([
			[
				'key1',
				{
					name: 'Deployment 1',
					description: 'Description 1'
				}
			],
			[
				'key2',
				{
					name: 'Deployment 2',
					description: 'Description 2'
				}
			]
		]);

		const getDeploymentDetails = vi.fn().mockReturnValue({ value: mockDeployments });

		setRegistry({ getDeploymentDetails });

		render(DeploymentsSection, {
			props: {
				orderName: 'Test Strategy'
			}
		});

		await waitFor(() => {
			expect(getDeploymentDetails).toHaveBeenCalledWith('Test Strategy');
		});

		await waitFor(() => {
			expect(screen.getByText('Deployment 1')).toBeInTheDocument();
			expect(screen.getByText('Deployment 2')).toBeInTheDocument();
		});
	});

	it('should handle error when fetching deployments fails', async () => {
		const testErrorMessage = 'Test error message';
		const getDeploymentDetails = vi.fn().mockReturnValue({
			error: { msg: testErrorMessage }
		});

		setRegistry({ getDeploymentDetails });

		render(DeploymentsSection, {
			props: {
				orderName: 'Test Strategy'
			}
		});

		await waitFor(() => {
			expect(getDeploymentDetails).toHaveBeenCalledWith('Test Strategy');
		});

		expect(await screen.findByText('Error loading deployments:')).toBeInTheDocument();
		const errorMessage = await screen.findByText(testErrorMessage);
		expect(errorMessage).toBeInTheDocument();
	});

	it('should fetch deployments when orderName prop changes', async () => {
		const getDeploymentDetails = vi.fn().mockReturnValue({ value: new Map() });
		setRegistry({ getDeploymentDetails });

		const { rerender } = render(DeploymentsSection, {
			props: {
				orderName: 'Initial Strategy'
			}
		});

		await waitFor(() => {
			expect(getDeploymentDetails).toHaveBeenCalledWith('Initial Strategy');
		});

		await rerender({ orderName: 'Updated Strategy' });

		await waitFor(() => {
			expect(getDeploymentDetails).toHaveBeenCalledWith('Updated Strategy');
		});

		expect(getDeploymentDetails).toHaveBeenCalledTimes(2);
	});
});
