import { describe, beforeEach, it, expect, vi } from 'vitest';
import { render, screen, waitFor } from '@testing-library/svelte';
import Page from './+page.svelte';
import { readable, writable, type Writable } from 'svelte/store';
import { useRegistry, type RegistryContext } from '@rainlanguage/ui-components';
import type { DotrainRegistry } from '@rainlanguage/orderbook';

vi.mock('@rainlanguage/ui-components', async (importOriginal) => {
	return {
		...((await importOriginal()) as object),
		useRegistry: vi.fn()
	};
});

describe('Deploy Page', () => {
	let registryStore: Writable<DotrainRegistry | null>;
	let loadingStore: Writable<boolean>;
	let errorStore: Writable<string | null>;
	let appendRegistryToHref: ReturnType<typeof vi.fn>;

	const createRegistry = (overrides?: Partial<DotrainRegistry>): DotrainRegistry => {
		return {
			getAllOrderDetails: vi.fn().mockReturnValue({
				error: null,
				value: new Map()
			}),
			getDeploymentDetails: vi.fn(),
			getGui: vi.fn(),
			getOrder: vi.fn(),
			getOrderDetail: vi.fn(),
			getValidOrders: vi.fn(),
			getInvalidOrders: vi.fn(),
			getRegistryUrl: vi.fn(),
			setRegistryUrl: vi.fn(),
			...overrides
		} as unknown as DotrainRegistry;
	};

	const setRegistry = (registry: DotrainRegistry | null) => {
		registryStore.set(registry);
	};

	beforeEach(() => {
		registryStore = writable<DotrainRegistry | null>(null);
		loadingStore = writable(false);
		errorStore = writable<string | null>(null);
		appendRegistryToHref = vi.fn((href: string) => href);

		vi.mocked(useRegistry).mockReturnValue({
			registry: registryStore,
			loading: loadingStore,
			error: errorStore,
			setRegistryUrl: vi.fn(),
			registryUrl: writable(''),
			isCustomRegistry: readable(false),
			appendRegistryToHref
		} as unknown as RegistryContext);
	});

	it('shows loading message while registry is loading', () => {
		loadingStore.set(true);

		render(Page);

		expect(screen.getByText('Loading orders…')).toBeInTheDocument();
	});

	it('shows registry initialization error', () => {
		errorStore.set('Registry unavailable');

		render(Page);

		expect(screen.getByText('Failed to initialize registry:')).toBeInTheDocument();
		expect(screen.getByTestId('error-message')).toHaveTextContent('Registry unavailable');
	});

	it('shows list error when fetching orders fails', async () => {
		const registry = createRegistry({
			getAllOrderDetails: vi.fn().mockReturnValue({
				error: { readableMsg: 'Failed to fetch orders' },
				value: null
			})
		});
		setRegistry(registry);

		render(Page);

		await waitFor(() => {
			expect(screen.getByText('Failed to load orders:')).toBeInTheDocument();
			expect(screen.getByTestId('error-message')).toHaveTextContent('Failed to fetch orders');
		});
	});

	it('shows fallback error message when readable message missing', async () => {
		const registry = createRegistry({
			getAllOrderDetails: vi.fn().mockReturnValue({
				error: { msg: 'Could not load' },
				value: null
			})
		});
		setRegistry(registry);

		render(Page);

		await waitFor(() => {
			expect(screen.getByTestId('error-message')).toHaveTextContent('Could not load');
		});
	});

	it('shows no orders message when registry returns empty map', async () => {
		const registry = createRegistry();
		setRegistry(registry);

		render(Page);

		await waitFor(() => {
			expect(screen.getByText('No orders found')).toBeInTheDocument();
		});
	});

	it('renders order tiles when orders are available', async () => {
		const ordersMap = new Map([
			['order1.dotrain', { name: 'Order One', short_description: 'First order' }],
			['order2.dotrain', { name: 'Order Two', short_description: 'Second order' }]
		]);
		const registry = createRegistry({
			getAllOrderDetails: vi.fn().mockReturnValue({
				error: null,
				value: ordersMap
			})
		});
		setRegistry(registry);

		render(Page);

		await waitFor(() => {
			expect(screen.getAllByTestId('order-short-tile')).toHaveLength(2);
		});
		expect(appendRegistryToHref).toHaveBeenCalledWith('/deploy/order1.dotrain');
		expect(appendRegistryToHref).toHaveBeenCalledWith('/deploy/order2.dotrain');
	});
});
