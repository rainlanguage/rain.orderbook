import { render, waitFor, screen } from '@testing-library/svelte';
import { vi, describe, it, expect, beforeEach } from 'vitest';
import Layout from './+layout.svelte';

const { mockPageStore, initialPageState } = await vi.hoisted(() => import('$lib/__mocks__/stores'));

const mockErcKit = vi.hoisted(() => ({
	init: vi.fn().mockResolvedValue(undefined)
}));
const mockDefaultConfig = vi.hoisted(() => vi.fn().mockReturnValue(mockErcKit));
const mockEnv = vi.hoisted(() => ({ browser: true }));

vi.mock('$app/stores', async (importOriginal) => {
	return {
		...((await importOriginal()) as object),
		page: mockPageStore
	};
});

vi.mock('$app/environment', () => mockEnv);

vi.mock('../lib/components/Sidebar.svelte', async () => {
	const MockSidebar = (await import('@rainlanguage/ui-components')).MockComponent;
	return { default: MockSidebar };
});

vi.mock('@rainlanguage/ui-components', async (importOriginal) => {
	const MockWalletProvider = (await import('../lib/__mocks__/MockComponent.svelte')).default;
	return {
		...(await importOriginal()),
		WalletProvider: MockWalletProvider
	};
});

vi.mock('$lib/components/TransactionProviderWrapper.svelte', async () => {
	const MockTransactionProviderWrapper = (await import('../lib/__mocks__/MockComponent.svelte'))
		.default;
	return {
		default: MockTransactionProviderWrapper
	};
});

vi.mock('$lib/stores/wagmi', () => ({
	defaultConfig: mockDefaultConfig,
	signerAddress: { subscribe: vi.fn() }
}));

vi.mock('$env/static/public', () => ({
	PUBLIC_WALLETCONNECT_PROJECT_ID: 'test-project-id'
}));

vi.mock('@wagmi/connectors', async (importOriginal) => {
	return {
		...(await importOriginal()),

		injected: vi.fn().mockReturnValue('injected-connector'),
		walletConnect: vi.fn().mockReturnValue('wallet-connect-connector')
	};
});

describe('Layout component', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		vi.resetAllMocks();
		mockDefaultConfig.mockReturnValue(mockErcKit);
		mockEnv.browser = true;
	});

	it('initializes wallet when in browser environment', async () => {
		const originalNavigator = global.navigator;

		Object.defineProperty(global, 'navigator', {
			value: {},
			writable: true
		});
		mockPageStore.mockSetSubscribeValue(initialPageState);

		render(Layout);

		expect(mockErcKit.init).toHaveBeenCalled();
		Object.defineProperty(global, 'navigator', {
			value: originalNavigator,
			writable: true
		});
	});

	it('displays an error message if wallet initialization fails', async () => {
		const originalNavigator = global.navigator;
		Object.defineProperty(global, 'navigator', {
			value: {},
			writable: true
		});

		mockErcKit.init.mockRejectedValue(new Error('Initialization failed'));
		mockPageStore.mockSetSubscribeValue(initialPageState);

		render(Layout);

		const errorMessage = await screen.findByText(
			'Failed to initialize wallet connection: Initialization failed. Please try again or check console.'
		);
		expect(errorMessage).toBeInTheDocument();

		Object.defineProperty(global, 'navigator', {
			value: originalNavigator,
			writable: true
		});
	});

	it('renders Homepage when on root path', async () => {
		mockPageStore.mockSetSubscribeValue({
			...initialPageState,
			url: new URL('http://localhost/')
		});

		const { container } = render(Layout);

		expect(container.querySelector('main')).not.toBeInTheDocument();
		expect(screen.getByTestId('homepage')).toBeInTheDocument();
	});

	it('renders Sidebar and main content when not on root path', async () => {
		mockPageStore.mockSetSubscribeValue({
			...initialPageState,
			url: new URL('http://localhost/some-page')
		});

		render(Layout);

		await waitFor(() => {
			expect(screen.getByTestId('layout-container')).toBeInTheDocument();
			expect(screen.getByTestId('mock-component')).toBeInTheDocument();
		});
	});

	it('does not initialize wallet when not in browser environment', async () => {
		const originalNavigator = global.navigator;
		mockEnv.browser = false;
		render(Layout);
		expect(mockErcKit.init).not.toHaveBeenCalled();
		Object.defineProperty(global, 'navigator', {
			value: originalNavigator,
			writable: true
		});
	});

	it('displays an error page if the page.error is set', async () => {
		mockPageStore.mockSetSubscribeValue({
			...initialPageState,
			data: {
				...initialPageState.data,
				errorMessage: 'Test error'
			}
		});
		render(Layout);

		expect(screen.getByText('Test error')).toBeInTheDocument();
		expect(screen.getByTestId('error-page')).toBeInTheDocument();
	});
});
