import { render, screen, waitFor } from '@testing-library/svelte';
import { writable, type Writable } from 'svelte/store';
import OrderPage from '../lib/components/deployment/OrderPage.svelte';
import { vi, describe, it, expect, beforeEach } from 'vitest';
import type { NameAndDescriptionCfg } from '@rainlanguage/orderbook';

type RegistryResult = {
	value?: Map<string, NameAndDescriptionCfg>;
	error?: { msg?: string; readableMsg?: string };
};

type MockRegistry = {
	getAllOrderDetails: () => RegistryResult;
};

type MockRegistryContext = {
	registry: Writable<MockRegistry | null>;
	loading: Writable<boolean>;
	error: Writable<string | null>;
	registryUrl: Writable<string>;
	isCustomRegistry: Writable<boolean>;
	setRegistryUrl: (url: string) => void;
	appendRegistryToHref: (href: string) => string;
};

const mockFetch = vi.fn();
vi.stubGlobal('fetch', mockFetch);

const createMockRegistryContext = (): MockRegistryContext => {
	const registry = writable<MockRegistry | null>(null);
	const loading = writable(false);
	const providerError = writable<string | null>(null);
	const registryUrl = writable('');
	const isCustomRegistry = writable(false);
	const setRegistryUrl = vi.fn();
	const appendRegistryToHref = vi.fn((href: string) => href);

	return {
		registry,
		loading,
		error: providerError,
		registryUrl,
		isCustomRegistry,
		setRegistryUrl,
		appendRegistryToHref
	};
};

let mockRegistryContext: MockRegistryContext = createMockRegistryContext();

vi.mock('$lib/providers/registry/useRegistry', () => ({
	useRegistry: () => mockRegistryContext
}));

vi.mock('../lib/components/deployment/DeploymentsSection.svelte', async () => {
	const MockDeploymentsSection = (await import('../lib/__mocks__/MockComponent.svelte')).default;
	return { default: MockDeploymentsSection };
});

const createRegistryWithDetails = (orderName: string, details: NameAndDescriptionCfg): MockRegistry => {
	return {
		getAllOrderDetails: () => ({
			value: new Map([[orderName, details]])
		})
	};
};

const createRegistryWithError = (errorMessage: string): MockRegistry => ({
	getAllOrderDetails: () => ({
		error: { msg: errorMessage }
	})
});

describe('OrderPage', () => {
	beforeEach(() => {
		mockRegistryContext = createMockRegistryContext();
		vi.clearAllMocks();
		mockFetch.mockReset();
	});

	it('renders order details when registry returns data', async () => {
		const orderName = 'TestOrder';
		mockRegistryContext.registry.set(
			createRegistryWithDetails(orderName, {
				name: 'Test Order',
				description: 'Test Description',
				short_description: 'Test Short Description'
			})
		);

		render(OrderPage, { props: { orderName } });

		await waitFor(() => {
			expect(screen.getByText('Test Order')).toBeInTheDocument();
			expect(screen.getByText('Test Description')).toBeInTheDocument();
		});
	});

	it('displays provider error message when registry fails to initialize', async () => {
		mockRegistryContext.error.set('Provider failure');

		render(OrderPage, { props: { orderName: 'AnyOrder' } });

		await waitFor(() => {
			expect(
				screen.getByText('Failed to initialize registry: Provider failure')
			).toBeInTheDocument();
		});
	});

	it('displays error message when registry returns an error response', async () => {
		const orderName = 'TestOrder';
		mockRegistryContext.registry.set(createRegistryWithError('Failed to get order details'));

		render(OrderPage, { props: { orderName } });

		await waitFor(() => {
			expect(screen.getByText('Error: Failed to get order details')).toBeInTheDocument();
		});
	});

	it('handles markdown fetch failure by showing an error message', async () => {
		const orderName = 'TestOrder';
		mockRegistryContext.registry.set(
			createRegistryWithDetails(orderName, {
				name: 'Test Order',
				description: 'https://example.com/description.md',
				short_description: 'Test Short Description'
			})
		);

		mockFetch.mockRejectedValueOnce(new Error('Failed to fetch'));

		render(OrderPage, { props: { orderName } });

		await waitFor(() => {
			expect(screen.getByText('Failed to fetch markdown')).toBeInTheDocument();
		});
	});

	it('renders markdown when description is a markdown URL', async () => {
		const orderName = 'TestOrder';
		mockRegistryContext.registry.set(
			createRegistryWithDetails(orderName, {
				name: 'Test Order',
				description: 'https://example.com/description.md',
				short_description: 'Test Short Description'
			})
		);

		mockFetch.mockResolvedValueOnce({
			ok: true,
			text: () => Promise.resolve('mock markdown content')
		});

		render(OrderPage, { props: { orderName } });

		await waitFor(() => {
			expect(screen.getByText('Test Order')).toBeInTheDocument();
			expect(screen.getByTestId('markdown-content')).toBeInTheDocument();
			expect(mockFetch).toHaveBeenCalledWith('https://example.com/description.md');
		});
	});

	it('falls back to plain text when markdown fetch response is not ok', async () => {
		const orderName = 'TestOrder';
		mockRegistryContext.registry.set(
			createRegistryWithDetails(orderName, {
				name: 'Test Order',
				description: 'https://example.com/description.md',
				short_description: 'Test Short Description'
			})
		);

		mockFetch.mockResolvedValueOnce({
			ok: false,
			statusText: 'Not Found'
		});

		render(OrderPage, { props: { orderName } });

		await waitFor(() => {
			expect(screen.getByText('Test Order')).toBeInTheDocument();
			expect(screen.getByTestId('plain-description')).toHaveTextContent(
				'https://example.com/description.md'
			);
		});
	});
});
