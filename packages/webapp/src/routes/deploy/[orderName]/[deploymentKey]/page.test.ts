import { render, screen, waitFor } from '@testing-library/svelte';
import { vi, describe, it, expect, beforeEach, afterEach } from 'vitest';
import DeployPage from './+page.svelte';
import { goto } from '$app/navigation';
import { useAccount, useToasts, useTransactions, useRegistry, type RegistryContext } from '@rainlanguage/ui-components';
import { readable, writable, type Writable } from 'svelte/store';
import type { DotrainOrderGui, DotrainRegistry } from '@rainlanguage/orderbook';

const { mockPageStore } = await vi.hoisted(() => import('@rainlanguage/ui-components'));

const { mockConnectedStore, mockAppKitModalStore } = await vi.hoisted(
	() => import('$lib/__mocks__/stores')
);

const mockPushGuiStateToUrlHistory = await vi.hoisted(() => vi.fn());

vi.mock('$app/stores', async (importOriginal) => {
	return {
		...((await importOriginal()) as object),
		page: mockPageStore
	};
});

vi.mock('$app/navigation', async (importOriginal) => {
	return {
		...((await importOriginal()) as object),
		goto: vi.fn()
	};
});

vi.mock('@rainlanguage/ui-components', async (importOriginal) => {
	const mockDeploymentSteps = (await import('$lib/__mocks__/MockComponent.svelte')).default;
	const mockGuiProvider = (await import('$lib/__mocks__/MockComponent.svelte')).default;
	return {
		...((await importOriginal()) as object),
		DeploymentSteps: mockDeploymentSteps,
		GuiProvider: mockGuiProvider,
		useTransactions: vi.fn(),
		useAccount: vi.fn(),
		useToasts: vi.fn(),
		useRegistry: vi.fn()
	};
});

vi.mock('$lib/stores/wagmi', () => ({
	connected: mockConnectedStore,
	appKitModal: mockAppKitModalStore
}));

vi.mock('$lib/services/modal', () => ({
	handleDisclaimerModal: vi.fn(),
	handleTransactionConfirmationModal: vi.fn()
}));

vi.mock('$lib/services/handleAddOrder', () => ({
	handleAddOrder: vi.fn()
}));

vi.mock('$lib/services/handleUpdateGuiState', () => ({
	pushGuiStateToUrlHistory: mockPushGuiStateToUrlHistory
}));

describe('DeployPage', () => {
	let registryStore: Writable<DotrainRegistry | null>;
	let loadingStore: Writable<boolean>;
	let errorStore: Writable<string | null>;
	let registryContext: RegistryContext;

	const setupRegistry = ({
		orderName,
		deploymentKey,
		getGuiResult,
		orderDetail,
		deploymentDetail
	}: {
		orderName?: string;
		deploymentKey?: string;
		getGuiResult?: { error: unknown; value: DotrainOrderGui | null };
		orderDetail?: unknown;
		deploymentDetail?: { name: string; description: string };
	}) => {
		const getAllOrderDetailsResult =
			orderName && orderDetail
				? {
						error: null,
						value: new Map([[orderName, orderDetail]])
					}
				: { error: null, value: new Map() };

		const getDeploymentDetailsResult =
			orderName && deploymentKey && deploymentDetail
				? {
						error: null,
						value: new Map([[deploymentKey, deploymentDetail]])
					}
				: { error: null, value: new Map() };

		const getAllOrderDetails = vi.fn().mockReturnValue(getAllOrderDetailsResult);
		const getDeploymentDetails = vi.fn().mockReturnValue(getDeploymentDetailsResult);
		const getGui = vi
			.fn()
			.mockResolvedValue(
				getGuiResult ?? { error: null, value: { dotrain: orderName } as unknown as DotrainOrderGui }
			);

		const registryValue = {
			getAllOrderDetails,
			getDeploymentDetails,
			getGui
		} as unknown as DotrainRegistry;
		registryStore.set(registryValue);

		return { getAllOrderDetails, getDeploymentDetails, getGui };
	};

	beforeEach(() => {
		vi.clearAllMocks();
		mockPageStore.reset();

		vi.mocked(useAccount).mockReturnValue({
			account: writable('0x123'),
			matchesAccount: vi.fn()
		});
		vi.mocked(useToasts).mockReturnValue({
			removeToast: vi.fn(),
			toasts: writable([]),
			addToast: vi.fn(),
			errToast: vi.fn()
		});
		vi.mocked(useTransactions).mockReturnValue({
			// @ts-expect-error simple object
			manager: writable({}),
			transactions: readable()
		});
		registryStore = writable<DotrainRegistry | null>(null);
		loadingStore = writable(false);
		errorStore = writable<string | null>(null);
		registryContext = {
			registry: registryStore,
			loading: loadingStore,
			error: errorStore,
			setRegistryUrl: vi.fn(),
			registryUrl: writable(''),
			isCustomRegistry: readable(false),
			appendRegistryToHref: vi.fn()
		} as unknown as RegistryContext;

		vi.mocked(useRegistry).mockReturnValue(registryContext);
	});

	afterEach(() => {
		vi.resetAllMocks();
	});

	it('should request GUI creation with URL state when registry data is available', async () => {
		const mockOrderName = 'order.dotrain';
		const mockDeploymentKey = 'deploy-key';
		const mockStateFromUrl = 'some-state';
		const { getGui } = setupRegistry({
			orderName: mockOrderName,
			deploymentKey: mockDeploymentKey,
			orderDetail: { name: 'Order', description: 'desc' },
			deploymentDetail: { name: 'Deployment', description: 'desc' }
		});

		mockPageStore.mockSetSubscribeValue({
			data: {
				orderDetail: {}
			},
			params: {
				orderName: mockOrderName,
				deploymentKey: mockDeploymentKey
			},
			url: new URL(`http://localhost:3000/deploy/${mockOrderName}/${mockDeploymentKey}?state=${mockStateFromUrl}`)
		});

		render(DeployPage);

		await waitFor(() => {
			expect(getGui).toHaveBeenCalledWith(
				mockOrderName,
				mockDeploymentKey,
				mockStateFromUrl,
				mockPushGuiStateToUrlHistory
			);
		});
	});

	it('should request GUI creation with null state when no state parameter is present', async () => {
		const mockOrderName = 'order.dotrain';
		const mockDeploymentKey = 'deploy-key';
		const { getGui } = setupRegistry({
			orderName: mockOrderName,
			deploymentKey: mockDeploymentKey,
			orderDetail: { name: 'Order', description: 'desc' },
			deploymentDetail: { name: 'Deployment', description: 'desc' }
		});

		mockPageStore.mockSetSubscribeValue({
			params: {
				orderName: mockOrderName,
				deploymentKey: mockDeploymentKey
			},
			url: new URL(`http://localhost:3000/deploy/${mockOrderName}/${mockDeploymentKey}`)
		});

		render(DeployPage);

		await waitFor(() => {
			expect(getGui).toHaveBeenCalledWith(
				mockOrderName,
				mockDeploymentKey,
				null,
				mockPushGuiStateToUrlHistory
			);
		});
	});

	it('should display loading indicator when registry is loading', async () => {
		loadingStore.set(true);

		render(DeployPage);

		expect(screen.getByText('Loading deployment…')).toBeInTheDocument();
	});

	it('should display registry error message when provided', async () => {
		errorStore.set('Registry unavailable');

		render(DeployPage);

		expect(screen.getByText('Failed to initialize registry: Registry unavailable')).toBeInTheDocument();
	});

	it('should show deployment not found message when registry lacks deployment', async () => {
		const mockOrderName = 'order.dotrain';
		const mockDeploymentKey = 'deploy-key';
		setupRegistry({
			orderName: mockOrderName,
			deploymentKey: mockDeploymentKey,
			orderDetail: { name: 'Order', description: 'desc' }
		});

		mockPageStore.mockSetSubscribeValue({
			params: {
				orderName: mockOrderName,
				deploymentKey: mockDeploymentKey
			},
			url: new URL(`http://localhost:3000/deploy/${mockOrderName}/${mockDeploymentKey}`)
		});

		render(DeployPage);

		await waitFor(() => {
			expect(
				screen.getByText('Deployment not found. Redirecting to deployments page...')
			).toBeInTheDocument();
		});
	});

	it('should redirect to /deploy if order parameters are missing', async () => {
		vi.useFakeTimers();

		render(DeployPage);

		expect(screen.getByText(/Deployment not found/i)).toBeInTheDocument();

		vi.advanceTimersByTime(5000);

		await vi.runAllTimersAsync();

		expect(goto).toHaveBeenCalledWith('/deploy');

		vi.useRealTimers();
	});

	it('should display error message when GUI creation fails', async () => {
		const mockOrderName = 'order.dotrain';
		const mockDeploymentKey = 'deploy-key';
		const errorMessage = 'Failed to build GUI';
		const { getGui } = setupRegistry({
			orderName: mockOrderName,
			deploymentKey: mockDeploymentKey,
			orderDetail: { name: 'Order', description: 'desc' },
			deploymentDetail: { name: 'Deployment', description: 'desc' },
			getGuiResult: {
				error: { readableMsg: errorMessage },
				value: null
			}
		});

		mockPageStore.mockSetSubscribeValue({
			params: {
				orderName: mockOrderName,
				deploymentKey: mockDeploymentKey
			},
			url: new URL(`http://localhost:3000/deploy/${mockOrderName}/${mockDeploymentKey}`)
		});

		render(DeployPage);

		await waitFor(() => {
			expect(getGui).toHaveBeenCalled();
			expect(screen.getByText(errorMessage)).toBeInTheDocument();
		});
	});

	it('should render GuiProvider when GUI creation succeeds', async () => {
		const mockOrderName = 'order.dotrain';
		const mockDeploymentKey = 'deploy-key';
		setupRegistry({
			orderName: mockOrderName,
			deploymentKey: mockDeploymentKey,
			orderDetail: { name: 'Order', description: 'desc' },
			deploymentDetail: { name: 'Deployment', description: 'desc' }
		});

		mockPageStore.mockSetSubscribeValue({
			params: {
				orderName: mockOrderName,
				deploymentKey: mockDeploymentKey
			},
			url: new URL(`http://localhost:3000/deploy/${mockOrderName}/${mockDeploymentKey}`)
		});

		render(DeployPage);

		await waitFor(() => {
			expect(screen.getByTestId('gui-provider')).toBeInTheDocument();
		});
	});
});
