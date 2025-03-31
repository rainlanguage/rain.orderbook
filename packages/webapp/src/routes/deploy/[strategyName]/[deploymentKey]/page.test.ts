import { render, screen } from '@testing-library/svelte';
import { vi, describe, it, expect, beforeEach, afterEach, type Mock } from 'vitest';
import DeployPage from './+page.svelte';
import { REGISTRY_URL } from '$lib/constants';
import * as handleGuiInitializationModule from '$lib/services/handleGuiInitialization';
import { goto } from '$app/navigation';

const {
	mockPageStore,
	mockWagmiConfigStore,
	mockConnectedStore,
	mockAppKitModalStore,
	mockSignerAddressStore
} = await vi.hoisted(() => import('$lib/__mocks__/stores'));

const mockPushState = vi.fn();
Object.defineProperty(window, 'history', {
	writable: true,
	configurable: true,
	value: { pushState: mockPushState }
});

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

vi.mock('@rainlanguage/ui-components', () => ({
	DeploymentSteps: vi.fn(),
	GuiProvider: vi.fn()
}));

vi.mock('$lib/stores/wagmi', () => ({
	wagmiConfig: mockWagmiConfigStore,
	connected: mockConnectedStore,
	appKitModal: mockAppKitModalStore,
	signerAddress: mockSignerAddressStore
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

	it('should add registry URL to search params if none exists', async () => {
		let pushStateSpy = vi.spyOn(window.history, 'pushState');
		// Ensure URL has no registry parameter
		mockPageStore.mockSetSubscribeValue({
			data: {
				stores: { settings: {} },
				dotrain: 'some dotrain',
				deployment: { key: 'test-key' },
				strategyDetail: {}
			},
			url: new URL('http://localhost:3000/deploy')
		});

		render(DeployPage);

		await vi.waitFor(() => {
			expect(pushStateSpy).toHaveBeenCalledWith(
				{},
				'',
				expect.stringContaining(`?registry=${REGISTRY_URL}`)
			);
		});
	});

	it('should not modify URL if registry param already exists', async () => {
		let pushStateSpy = vi.spyOn(window.history, 'pushState');
		const customRegistryUrl = 'https://custom-registry.example.com';
		mockPageStore.mockSetSubscribeValue({
			data: {
				stores: { settings: {} },
				dotrain: null as unknown as string,
				deployment: null as unknown as { key: string },
				strategyDetail: {}
			},
			url: new URL(`http://localhost:3000/deploy?registry=${customRegistryUrl}`)
		});

		render(DeployPage);

		await vi.waitFor(() => {
			expect(pushStateSpy).not.toHaveBeenCalled();
		});
	});

	it('should redirect to /deploy if dotrain or deployment is missing', async () => {
		// Use fake timers before component rendering
		vi.useFakeTimers();

		// Set missing dotrain and deployment
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

		// Fast-forward time
		vi.advanceTimersByTime(5000);

		await vi.runAllTimersAsync();

		expect(goto).toHaveBeenCalledWith('/deploy');

		vi.useRealTimers();
	});

	it('should display error message when GUI initialization fails', async () => {
		const errorMessage = 'Failed to initialize GUI';

		mockPageStore.mockSetSubscribeValue({
			data: {
				stores: { settings: {} },
				dotrain: 'some dotrain',
				deployment: { key: 'test-key' },
				strategyDetail: {}
			},
			url: new URL('http://localhost:3000/deploy/strategy/key')
		});

		vi.mocked(handleGuiInitializationModule.handleGuiInitialization).mockResolvedValue({
			gui: null,
			error: errorMessage
		});

		render(DeployPage);

		await vi.waitFor(() => {
			expect(screen.getByText(errorMessage)).toBeInTheDocument();
		});
	});
});
