import { render, screen, waitFor } from '@testing-library/svelte';
import { vi, describe, it, expect, beforeEach, afterEach } from 'vitest';
import DeployPage from './+page.svelte';
import * as handleGuiInitializationModule from '$lib/services/handleGuiInitialization';
import { goto } from '$app/navigation';
import { useAccount, useToasts, useTransactions } from '@rainlanguage/ui-components';
import { readable, writable } from 'svelte/store';

const { mockPageStore } = await vi.hoisted(() => import('@rainlanguage/ui-components'));

const { mockConnectedStore, mockAppKitModalStore } = await vi.hoisted(
	() => import('$lib/__mocks__/stores')
);

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
	return {
		...((await importOriginal()) as object),
		DeploymentSteps: mockDeploymentSteps,
		useTransactions: vi.fn(),
		useAccount: vi.fn(),
		useToasts: vi.fn()
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

vi.mock('$lib/services/handleGuiInitialization', () => ({
	handleGuiInitialization: vi.fn().mockResolvedValue({
		gui: null,
		error: null
	})
}));

vi.mock('$lib/services/handleAddOrder', () => ({
	handleAddOrder: vi.fn()
}));

describe('DeployPage', () => {
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
		vi.mocked(handleGuiInitializationModule.handleGuiInitialization).mockResolvedValue({
			gui: null,
			error: null
		});
	});

	afterEach(() => {
		vi.resetAllMocks();
	});

	it('should call handleGuiInitialization with correct parameters when dotrain and deployment exist', async () => {
		const mockDotrain = 'mock-dotrain';
		const mockDeploymentKey = 'test-key';
		const mockStateFromUrl = 'some-state';

		mockPageStore.mockSetSubscribeValue({
			data: {
				dotrain: mockDotrain,
				deployment: { key: mockDeploymentKey },
				orderDetail: {}
			},
			url: new URL(`http://localhost:3000/deploy?state=${mockStateFromUrl}`)
		});

		render(DeployPage);

		await vi.waitFor(() => {
			expect(handleGuiInitializationModule.handleGuiInitialization).toHaveBeenCalledWith(
				mockDotrain,
				mockDeploymentKey,
				mockStateFromUrl
			);
		});
	});

	it('should not call handleGuiInitialization when dotrain is missing', async () => {
		mockPageStore.mockSetSubscribeValue({
			data: {
				dotrain: null as unknown as string,
				deployment: { key: 'test-key' },
				orderDetail: {}
			},
			url: new URL('http://localhost:3000/deploy')
		});

		render(DeployPage);

		await new Promise((resolve) => setTimeout(resolve, 50));

		expect(handleGuiInitializationModule.handleGuiInitialization).not.toHaveBeenCalled();
	});

	it('should not call handleGuiInitialization when deployment is missing', async () => {
		mockPageStore.mockSetSubscribeValue({
			data: {
				dotrain: 'some-dotrain',
				deployment: null as unknown as { key: string },
				orderDetail: {}
			},
			url: new URL('http://localhost:3000/deploy')
		});

		render(DeployPage);

		await new Promise((resolve) => setTimeout(resolve, 50));

		expect(handleGuiInitializationModule.handleGuiInitialization).not.toHaveBeenCalled();
	});

	it('should redirect to /deploy if dotrain or deployment is missing', async () => {
		vi.useFakeTimers();

		mockPageStore.mockSetSubscribeValue({
			data: {
				dotrain: null as unknown as string,
				deployment: null as unknown as { key: string },
				orderDetail: {}
			},
			url: new URL('http://localhost:3000/deploy/order/key')
		});

		render(DeployPage);

		expect(screen.getByText(/Deployment not found/i)).toBeInTheDocument();

		vi.advanceTimersByTime(5000);

		await vi.runAllTimersAsync();

		expect(goto).toHaveBeenCalledWith('/deploy');

		vi.useRealTimers();
	});

	it('should show error message when GUI initialization fails', async () => {
		mockPageStore.mockSetSubscribeValue({
			data: {
				dotrain: 'https://dotrain.example.com',
				deployment: {
					key: 'test-deployment'
				},
				orderDetail: {}
			},
			url: new URL('http://localhost:3000/deploy')
		});

		const errorMessage = 'Failed to initialize GUI';
		vi.mocked(handleGuiInitializationModule.handleGuiInitialization).mockResolvedValue({
			gui: null,
			error: errorMessage
		});

		render(DeployPage);

		await waitFor(() => {
			expect(screen.getByText(errorMessage)).toBeInTheDocument();
		});
	});

	it('should handle initialization with empty state from URL', async () => {
		mockPageStore.mockSetSubscribeValue({
			data: {
				dotrain: 'https://dotrain.example.com',
				deployment: {
					key: 'test-deployment'
				},
				orderDetail: {}
			},
			url: new URL('http://localhost:3000/deploy')
		});

		render(DeployPage);

		await waitFor(() => {
			expect(handleGuiInitializationModule.handleGuiInitialization).toHaveBeenCalledWith(
				'https://dotrain.example.com',
				'test-deployment',
				''
			);
		});
	});

	it('should correctly pass state parameter from URL to handleGuiInitialization', async () => {
		const stateValue = 'someEncodedStateFromUrl';

		vi.clearAllMocks();

		mockPageStore.mockSetSubscribeValue({
			data: {
				dotrain: 'https://dotrain.example.com',
				deployment: {
					key: 'test-deployment'
				},
				orderDetail: {}
			},
			url: new URL(`http://localhost:3000/deploy?state=${stateValue}`)
		});

		render(DeployPage);

		await vi.waitFor(() => {
			expect(handleGuiInitializationModule.handleGuiInitialization).toHaveBeenCalledWith(
				'https://dotrain.example.com',
				'test-deployment',
				stateValue
			);
		});
	});
});
