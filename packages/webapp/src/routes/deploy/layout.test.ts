import { describe, expect, it, beforeEach, vi } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import Layout from './+layout.svelte';
import RegistryManager from '$lib/services/RegistryManager';
import type { Mock } from 'vitest';
import { load } from './+layout';
import { REGISTRY_URL } from '$lib/constants';
import { fetchRegistryDotrains, validateStrategies } from '@rainlanguage/ui-components/services';

const { mockPageStore } = await vi.hoisted(() => import('$lib/__mocks__/stores'));

vi.mock('$app/stores', () => {
	return { page: mockPageStore };
});

vi.mock('$lib/services/RegistryManager', () => ({
	default: {
		setToStorage: vi.fn(),
		clearFromStorage: vi.fn(),
		isCustomRegistry: vi.fn(),
		getFromStorage: vi.fn(),
		updateUrlParam: vi.fn()
	}
}));

vi.mock('@rainlanguage/ui-components/services', () => ({
	validateStrategies: vi.fn().mockResolvedValue({
		validStrategies: [],
		invalidStrategies: []
	}),
	fetchRegistryDotrains: vi.fn().mockResolvedValue([])
}));

describe('Layout Component', () => {
	beforeEach((): void => {
		vi.clearAllMocks();
		mockPageStore.reset();
		vi.unstubAllGlobals();
		vi.stubGlobal('localStorage', {
			getItem: vi.fn().mockReturnValue(null)
		});
	});

	it('should show advanced mode toggle on deploy page', () => {
		mockPageStore.mockSetSubscribeValue({
			url: {
				pathname: '/deploy'
			} as unknown as URL
		});

		render(Layout);

		expect(screen.getByText('Advanced mode')).toBeInTheDocument();
	});

	it('should not show advanced mode toggle on non-deploy pages', () => {
		mockPageStore.mockSetSubscribeValue({
			url: {
				pathname: '/other-page'
			} as unknown as URL
		});

		render(Layout);

		expect(screen.queryByText('Advanced mode')).toBeNull();
	});

	it('should show custom registry warning when using non-default registry', () => {
		(RegistryManager.getFromStorage as Mock).mockReturnValue('https://custom-registry.com');
		(RegistryManager.isCustomRegistry as Mock).mockReturnValue(true);

		mockPageStore.mockSetSubscribeValue({
			url: {
				pathname: '/deploy'
			} as unknown as URL
		});

		render(Layout);

		expect(RegistryManager.getFromStorage).toHaveBeenCalled();
		expect(screen.getByTestId('custom-registry-warning')).toBeTruthy();
	});

	it('should display InputRegistryUrl when advanced mode is on', () => {
		vi.stubGlobal('localStorage', {
			getItem: vi.fn().mockReturnValue('https://custom-registry.com')
		});

		mockPageStore.mockSetSubscribeValue({
			url: {
				pathname: '/deploy'
			} as unknown as URL
		});

		render(Layout);

		expect(screen.getByTestId('registry-input')).toBeTruthy();
	});

	it('should not display InputRegistryUrl when advanced mode is off', () => {
		vi.stubGlobal('localStorage', {
			getItem: vi.fn().mockReturnValue(null)
		});

		mockPageStore.mockSetSubscribeValue({
			url: {
				pathname: '/deploy'
			} as unknown as URL
		});

		render(Layout);

		expect(screen.queryByTestId('registry-input')).toBeNull();
	});
});

describe('Layout Load Function', () => {
	beforeEach((): void => {
		vi.clearAllMocks();
	});

	it('should store custom registry from URL in storage', async () => {
		const customRegistry = 'https://custom-registry.com';
		const mockEvent = {
			url: new URL(`https://example.com/deploy?registry=${customRegistry}`)
		};

		(RegistryManager.isCustomRegistry as Mock).mockReturnValue(true);

		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		await load(mockEvent as any);

		expect(RegistryManager.setToStorage).toHaveBeenCalledWith(customRegistry);
	});

	it('should clear registry from storage when using default registry', async () => {
		const mockEvent = {
			url: new URL('https://example.com/deploy')
		};

		(RegistryManager.isCustomRegistry as Mock).mockReturnValue(false);

		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		await load(mockEvent as any);

		expect(RegistryManager.clearFromStorage).toHaveBeenCalled();
	});

	it('should return registry and strategies data on successful fetch', async () => {
		const mockEvent = {
			url: new URL('https://example.com/deploy')
		};

		const mockRegistryDotrains = [{ id: 'test-dotrain' }];
		const mockValidStrategies = [{ id: 'valid-strategy' }];
		const mockInvalidStrategies = [{ id: 'invalid-strategy' }];

		(fetchRegistryDotrains as Mock).mockResolvedValue(mockRegistryDotrains);
		(validateStrategies as Mock).mockResolvedValue({
			validStrategies: mockValidStrategies,
			invalidStrategies: mockInvalidStrategies
		});

		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		const result = await load(mockEvent as any);

		expect(result).toEqual({
			registry: REGISTRY_URL,
			registryDotrains: mockRegistryDotrains,
			validStrategies: mockValidStrategies,
			invalidStrategies: mockInvalidStrategies,
			error: null
		});
	});

	it('should return error information when fetch fails', async () => {
		const mockEvent = {
			url: new URL('https://example.com/deploy')
		};

		const mockError = new Error('Failed to fetch');

		(fetchRegistryDotrains as Mock).mockRejectedValue(mockError);

		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		const result = await load(mockEvent as any);

		expect(result).toEqual({
			registry: REGISTRY_URL,
			registryDotrains: [],
			validStrategies: [],
			invalidStrategies: [],
			error: 'Failed to fetch'
		});
	});
});
