import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/svelte';
import Page from './+page.svelte';
import { handleGuiInitialization } from '$lib/services/handleGuiInitialization';
import { goto } from '$app/navigation';
import { readable } from 'svelte/store';
import type { Page as SvelteKitPage } from '@sveltejs/kit';
import type { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';

const { mockPageStore } = await vi.hoisted(() => import('$lib/__mocks__/stores'));

vi.mock('$app/stores', () => ({
	page: mockPageStore
}));

vi.mock('$app/navigation', () => ({
	goto: vi.fn()
}));

vi.mock('$lib/stores/wagmi', () => ({
	connected: true,
	appKitModal: {},
	signerAddress: 'test-address',
	wagmiConfig: {}
}));

vi.mock('$lib/services/handleGuiInitialization', () => ({
	handleGuiInitialization: vi.fn()
}));

vi.mock('$lib/services/modal', () => ({
	handleDeployModal: vi.fn(),
	handleDisclaimerModal: vi.fn()
}));

vi.mock('@rainlanguage/ui-components', async () => {
	const MockComponent = (await import('$lib/__mocks__/MockComponent.svelte')).default;
	return { GuiProvider: MockComponent, DeploymentSteps: MockComponent };
});

describe('+page.svelte', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		mockPageStore.mockSetSubscribeValue({
			data: {
				stores: { settings: readable({}) },
				dotrain: 'https://dotrain.example.com',
				deployment: {
					key: 'test-deployment',
					name: 'Test Deployment',
					description: 'This is a test deployment'
				},
				strategyDetail: {
					name: 'Test Strategy',
					description: 'This is a test strategy'
				}
			},
			url: {
				searchParams: {
					get: vi.fn().mockReturnValue('')
				} as unknown as {
					get: (key: string) => string | null;
				}
			}
		} as unknown as SvelteKitPage);
	});

	it('should show loading message and redirect when dotrain or deployment is not found', async () => {
		vi.useFakeTimers();

		mockPageStore.mockSetSubscribeValue({
			data: {
				stores: { settings: readable({}) },
				dotrain: null,
				deployment: null,
				strategyDetail: null
			},
			url: {
				searchParams: {
					get: vi.fn().mockReturnValue('')
				} as unknown as {
					get: (key: string) => string | null;
				}
			}
		} as unknown as SvelteKitPage);

		render(Page);

		expect(screen.getByText(/Deployment not found/i)).toBeInTheDocument();
		vi.advanceTimersByTime(5000);

		expect(goto).toHaveBeenCalledWith('/deploy');
		vi.useRealTimers();
	});

	it('should initialize GUI and render DeploymentSteps when GUI is available', async () => {
		const mockGui = {
			name: 'Test GUI'
		};

		mockPageStore.mockSetSubscribeValue({
			data: {
				stores: { settings: readable({}) },
				dotrain: 'https://dotrain.example.com',
				deployment: {
					key: 'test-deployment',
					name: 'Test Deployment',
					description: 'This is a test deployment'
				},
				strategyDetail: {
					name: 'Test Strategy',
					description: 'This is a test strategy'
				}
			},
			url: {
				searchParams: {
					get: vi.fn().mockReturnValue('test-state')
				} as unknown as {
					get: (key: string) => string | null;
				}
			}
		} as unknown as SvelteKitPage);

		vi.mocked(handleGuiInitialization).mockResolvedValue({
			gui: mockGui as unknown as DotrainOrderGui,
			error: null
		});

		render(Page);

		await waitFor(() => {
			expect(handleGuiInitialization).toHaveBeenCalledWith(
				'https://dotrain.example.com',
				'test-deployment',
				'test-state'
			);
		});
	});

	it('should show error message when GUI initialization fails', async () => {
		vi.mocked(handleGuiInitialization).mockResolvedValue({
			gui: null,
			error: 'Failed to initialize GUI'
		});

		render(Page);

		await waitFor(() => {
			expect(screen.getByText('Failed to initialize GUI')).toBeInTheDocument();
		});
	});

	it('should not call handleGuiInitialization when dotrain is null', async () => {
		mockPageStore.mockSetSubscribeValue({
			data: {
				stores: { settings: readable({}) },
				dotrain: null,
				deployment: {
					key: 'test-deployment',
					name: 'Test Deployment',
					description: 'This is a test deployment'
				},
				strategyDetail: {
					name: 'Test Strategy',
					description: 'This is a test strategy'
				}
			},
			url: {
				searchParams: {
					get: vi.fn().mockReturnValue('test-state')
				} as unknown as {
					get: (key: string) => string | null;
				}
			}
		} as unknown as SvelteKitPage);

		render(Page);

		await new Promise((resolve) => setTimeout(resolve, 0));

		expect(handleGuiInitialization).not.toHaveBeenCalled();
	});

	it('should not call handleGuiInitialization when deployment is null', async () => {
		mockPageStore.mockSetSubscribeValue({
			data: {
				stores: { settings: readable({}) },
				dotrain: 'https://dotrain.example.com',
				deployment: null,
				strategyDetail: {
					name: 'Test Strategy',
					description: 'This is a test strategy'
				}
			},
			url: {
				searchParams: {
					get: vi.fn().mockReturnValue('test-state')
				} as unknown as {
					get: (key: string) => string | null;
				}
			}
		} as unknown as SvelteKitPage);

		render(Page);

		await new Promise((resolve) => setTimeout(resolve, 0));

		expect(handleGuiInitialization).not.toHaveBeenCalled();
	});

	it('should handle initialization with empty state from URL', async () => {
		mockPageStore.mockSetSubscribeValue({
			data: {
				stores: { settings: readable({}) },
				dotrain: 'https://dotrain.example.com',
				deployment: {
					key: 'test-deployment'
				},
				strategyDetail: {
					name: 'Test Strategy'
				}
			},
			url: {
				searchParams: {
					get: vi.fn().mockReturnValue('')
				} as unknown as {
					get: (key: string) => string | null;
				}
			}
		} as unknown as SvelteKitPage);

		const mockGui = { name: 'Test GUI' } as unknown as DotrainOrderGui;
		vi.mocked(handleGuiInitialization).mockResolvedValue({
			gui: mockGui,
			error: null
		});

		render(Page);

		await waitFor(() => {
			expect(handleGuiInitialization).toHaveBeenCalledWith(
				'https://dotrain.example.com',
				'test-deployment',
				''
			);
		});
	});
});
