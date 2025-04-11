import { REGISTRY_URL } from '$lib/constants';
import type { LayoutLoad } from './$types';

export const load: LayoutLoad = async ({ url }) => {
	const registryFromUrl = url.searchParams.get('registry') || REGISTRY_URL;

	return {
		registryFromUrl
	};
};

if (import.meta.vitest) {
	const { describe, it, expect } = import.meta.vitest;

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


		it('should should pass default registry url when no registry param is provided', async () => {
			const result = await load(createUrlMock(null));

			expect(result).toEqual({
				registryFromUrl: REGISTRY_URL
			});
		});

		it('should should pass custom registry url when registry param is provided', async () => {
			const result = await load(createUrlMock('https://custom.registry.url'));

			expect(result).toEqual({
				registryFromUrl: 'https://custom.registry.url'
			});
		});
	});
}
