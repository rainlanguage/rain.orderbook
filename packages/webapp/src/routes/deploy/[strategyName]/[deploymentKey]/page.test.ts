import { render } from '@testing-library/svelte';
import { vi, describe, it, expect, beforeEach, afterEach } from 'vitest';
import DeployPage from './+page.svelte';
import * as handleGuiInitializationModule from '$lib/services/handleGuiInitialization';
import { goto } from '$app/navigation';

const {
	mockPageStore,
	mockWagmiConfigStore,
	mockConnectedStore,
	mockAppKitModalStore,
	mockSignerAddressStore
} = await vi.hoisted(() => import('$lib/__mocks__/stores'));

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

		vi.advanceTimersByTime(5000);

		await vi.runAllTimersAsync();

		expect(goto).toHaveBeenCalledWith('/deploy');

		vi.useRealTimers();
	});
});
