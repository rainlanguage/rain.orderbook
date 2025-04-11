import { render, screen, waitFor } from '@testing-library/svelte';
import { vi, describe, it, expect, beforeEach, afterEach } from 'vitest';
import DeployPage from './+page.svelte';
import * as handleGuiInitializationModule from '$lib/services/handleGuiInitialization';
import { goto } from '$app/navigation';

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
		DeploymentSteps: mockDeploymentSteps
	};
});

vi.mock('$lib/stores/wagmi', () => ({
	connected: mockConnectedStore,
	appKitModal: mockAppKitModalStore
}));

vi.mock('$lib/services/modal', () => ({
	handleDeployModal: vi.fn(),
	handleDisclaimerModal: vi.fn()
}));

vi.mock('$lib/services/handleGuiInitialization', () => ({
	handleGuiInitialization: vi.fn().mockResolvedValue({
		gui: null,
		error: null
	})
}));

describe('DeployPage', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		mockPageStore.reset();

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
				stores: { settings: {} },
				dotrain: mockDotrain,
				deployment: { key: mockDeploymentKey },
				strategyDetail: {}
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
				stores: { settings: {} },
				dotrain: null as unknown as string,
				deployment: { key: 'test-key' },
				strategyDetail: {}
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
				stores: { settings: {} },
				dotrain: 'some-dotrain',
				deployment: null as unknown as { key: string },
				strategyDetail: {}
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
				stores: { settings: {} },
				dotrain: null as unknown as string,
				deployment: null as unknown as { key: string },
				strategyDetail: {}
			},
			url: new URL('http://localhost:3000/deploy/strategy/key')
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
				stores: { settings: {} },
				dotrain: 'https://dotrain.example.com',
				deployment: {
					key: 'test-deployment'
				},
				strategyDetail: {}
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
				stores: { settings: {} },
				dotrain: 'https://dotrain.example.com',
				deployment: {
					key: 'test-deployment'
				},
				strategyDetail: {}
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
				stores: { settings: {} },
				dotrain: 'https://dotrain.example.com',
				deployment: {
					key: 'test-deployment'
				},
				strategyDetail: {}
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
