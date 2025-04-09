import { describe, expect, it, beforeEach, vi } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import Layout from './+layout.svelte';
import RegistryManager from '$lib/services/RegistryManager';
import type { Mock } from 'vitest';
import { load } from './+layout';
import { REGISTRY_URL } from '$lib/constants';
import { fetchRegistryDotrains, validateStrategies, type RegistryDotrain } from '@rainlanguage/ui-components/services';
import type { ValidStrategyDetail } from '@rainlanguage/ui-components';
import type { InvalidStrategyDetail } from '@rainlanguage/ui-components';

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

vi.mock('@rainlanguage/ui-components/services', () => ({
	validateStrategies: vi.fn(),
	fetchRegistryDotrains: vi.fn()
}));

type LoadResult = {
	registry: string;
	registryDotrains: RegistryDotrain[];
	validStrategies: ValidStrategyDetail[];
	invalidStrategies: InvalidStrategyDetail[];
	error: string | null;
};

const mockDotrains = ['dotrain1', 'dotrain2'] as unknown as RegistryDotrain[];
const mockValidated = {
	validStrategies: ['strategy1', 'strategy2'] as unknown as ValidStrategyDetail[],
	invalidStrategies: ['invalidStrategy'] as unknown as InvalidStrategyDetail[]
};



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
});

describe('Layout load function', () => {
	beforeEach(() => {
		vi.resetAllMocks();
	});

	const createUrlMock = (registryParam: string | null) =>
		({
			url: {
				searchParams: {
					get: vi.fn().mockReturnValue(registryParam)
				}
			}
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
		}) as any;

	it('should load strategies from default registry URL when no registry param is provided', async () => {
		(validateStrategies as Mock).mockResolvedValue(mockValidated);
		(fetchRegistryDotrains as Mock).mockResolvedValue(mockDotrains);

		const result = await load(createUrlMock(null));

		expect(fetchRegistryDotrains).toHaveBeenCalledWith(REGISTRY_URL);

		expect(validateStrategies).toHaveBeenCalledWith(mockDotrains);

		expect(result).toEqual({
			registry: REGISTRY_URL,
			registryDotrains: mockDotrains,
			validStrategies: mockValidated.validStrategies,
			invalidStrategies: mockValidated.invalidStrategies,
			error: null
		});
	});

	it('should load strategies from custom registry URL when registry param is provided', async () => {
		const customRegistry = 'https://custom.registry.url';
		(fetchRegistryDotrains as Mock).mockResolvedValue(mockDotrains);
		(validateStrategies as Mock).mockResolvedValue(mockValidated);

		const result = await load(createUrlMock(customRegistry));

		expect(result).toEqual({
			registry: customRegistry,
			registryDotrains: mockDotrains,
			validStrategies: mockValidated.validStrategies,
			invalidStrategies: mockValidated.invalidStrategies,
			error: null
		});
	});

	it('should handle errors when fetchRegistryDotrains fails', async () => {
		const errorMessage = 'Failed to fetch registry dotrains';
		(fetchRegistryDotrains as Mock).mockRejectedValue(new Error(errorMessage));
		const result = await load(createUrlMock(null));

		expect(validateStrategies).not.toHaveBeenCalled();

		expect(result).toEqual({
			registry: REGISTRY_URL,
			registryDotrains: [],
			validStrategies: [],
			invalidStrategies: [],
			error: errorMessage
		});

		const typedResult = result as LoadResult;
		if (typedResult.error) {
			expect(typedResult.error).toBe(errorMessage);
		}
	});

	it('should handle errors when validateStrategies fails', async () => {
		const errorMessage = 'Failed to validate strategies';
		(validateStrategies as Mock).mockRejectedValue(new Error(errorMessage));
		const result = await load(createUrlMock(null));

		expect(result).toEqual({
			registry: REGISTRY_URL,
			registryDotrains: [],
			validStrategies: [],
			invalidStrategies: [],
			error: errorMessage
		});
	});

	it('should handle non-Error exceptions with an "Unknown error" message', async () => {
		(fetchRegistryDotrains as Mock).mockRejectedValue('Not an error object');
		const result = await load(createUrlMock(null));

		expect(result).toEqual({
			registry: REGISTRY_URL,
			registryDotrains: [],
			validStrategies: [],
			invalidStrategies: [],
			error: 'Unknown error occurred'
		});
	});

	it('should handle when fetchRegistryDotrains and validateStrategies return empty arrays', async () => {
		const emptyDotrains: RegistryDotrain[] = [];
		const emptyValidated = {
			validStrategies: [] as ValidStrategyDetail[],
			invalidStrategies: [] as InvalidStrategyDetail[]
		};

		(fetchRegistryDotrains as Mock).mockResolvedValue(emptyDotrains);
		(validateStrategies as Mock).mockResolvedValue(emptyValidated);

		const result = await load(createUrlMock(null));

		expect(fetchRegistryDotrains).toHaveBeenCalledWith(REGISTRY_URL);
		expect(validateStrategies).toHaveBeenCalledWith(emptyDotrains);

		expect(result).toEqual({
			registry: REGISTRY_URL,
			registryDotrains: emptyDotrains,
			validStrategies: emptyValidated.validStrategies,
			invalidStrategies: emptyValidated.invalidStrategies,
			error: null
		});
	});
});
