import { REGISTRY_URL } from '$lib/constants';
import {
	validateStrategies,
	fetchRegistryDotrains,
	type RegistryDotrain
} from '@rainlanguage/ui-components/services';
import type { LayoutLoad } from './$types';
import type { Mock } from 'vitest';
import type { InvalidStrategyDetail, ValidStrategyDetail } from '@rainlanguage/ui-components';

type LoadResult = {
	registryFromUrl: string;
};

export const load: LayoutLoad = async ({ url }) => {
	const registryFromUrl = url.searchParams.get('registry') || REGISTRY_URL;

	return {
		registryFromUrl
	};
};

if (import.meta.vitest) {
	const { describe, it, expect } = import.meta.vitest;
	const mockDotrains = ['dotrain1', 'dotrain2'] as unknown as RegistryDotrain[];
	const mockValidated = {
		validStrategies: ['strategy1', 'strategy2'] as unknown as ValidStrategyDetail[],
		invalidStrategies: ['invalidStrategy'] as unknown as InvalidStrategyDetail[]
	};

	vi.mock('@rainlanguage/ui-components/services', () => ({
		validateStrategies: vi.fn(),
		fetchRegistryDotrains: vi.fn()
	}));

	vi.mock('$lib/services/RegistryManager', () => ({
		default: {
			isCustomRegistry: vi.fn(),
			setToStorage: vi.fn(),
			clearFromStorage: vi.fn()
		}
	}));

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
}
