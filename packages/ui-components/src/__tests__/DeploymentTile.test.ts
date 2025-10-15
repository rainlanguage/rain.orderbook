import { render, screen } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import DeploymentTile from '../lib/components/deployment/DeploymentTile.svelte';
import { writable, readable } from 'svelte/store';
import { useRegistry } from '$lib/providers/registry/useRegistry';

// Mock the goto function
vi.mock('$app/navigation', () => ({
	goto: vi.fn()
}));

vi.mock('$lib/providers/registry/useRegistry', () => ({
	useRegistry: vi.fn()
}));

describe('DeploymentTile', () => {
	const mockProps = {
		orderName: 'test-order',
		key: 'test-key',
		name: 'Test Deployment',
		description: 'This is a test deployment description'
	};

	beforeEach(() => {
		vi.mocked(useRegistry).mockReturnValue({
			registry: writable(null),
			registryUrl: writable(''),
			loading: writable(false),
			error: writable(null),
			isCustomRegistry: readable(false),
			setRegistryUrl: vi.fn<(url: string) => void>(),
			appendRegistryToHref: (href: string) => href
		});
	});

	it('renders the deployment name and description', () => {
		render(DeploymentTile, mockProps);

		expect(screen.getByText('Test Deployment')).toBeInTheDocument();
		expect(screen.getByText('This is a test deployment description')).toBeInTheDocument();
	});
});
