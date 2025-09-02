import { getContext, setContext } from 'svelte';
import type { DotrainRegistry } from '@rainlanguage/orderbook';
import type { Readable, Writable } from 'svelte/store';

export const REGISTRY_KEY = 'registry_key';
/**
 * Context shape for the DotrainRegistry provider
 */
export type RegistryContext = {
    registry: Writable<DotrainRegistry | null>;
    loading: Writable<boolean>;
    error: Writable<string | null>;
    setRegistryUrl: (url: string) => void;
    registryUrl: Writable<string>;
    isCustomRegistry: Readable<boolean>;
    appendRegistryToHref: (href: string) => string;
};

/**
 * Retrieves the registry context directly from Svelte's context
 */
export const getRegistryContext = (): RegistryContext => {
    const ctx = getContext<RegistryContext>(REGISTRY_KEY);
    if (!ctx) {
        throw new Error(
            'No registry provider was found in Svelte context. Did you forget to wrap your component with RegistryProvider?'
        );
    }
    return ctx;
};

/**
 * Sets the registry manager in Svelte's context
 */
export const setRegistryContext = (context: RegistryContext) => {
    setContext(REGISTRY_KEY, context);
};

if (import.meta.vitest) {
	const { describe, it, expect, vi, beforeEach } = import.meta.vitest;

    vi.mock('svelte', async (importOriginal) => ({
        ...((await importOriginal()) as object),
        getContext: vi.fn()
    }));

	describe('getRegistryContext', () => {
		const mockGetContext = vi.mocked(getContext);

		beforeEach(() => {
			mockGetContext.mockReset();
		});

        it('should return the registry context when it exists', () => {
            const mockRegistry = {
                registry: null,
                loading: false,
                error: null,
                setRegistryUrl: () => {},
                registryUrl: { subscribe: () => () => {} },
                isCustomRegistry: { subscribe: () => () => {} },
                appendRegistryToHref: (href: string) => href
            } as unknown as RegistryContext;

            mockGetContext.mockImplementation((key) => {
                if (key === REGISTRY_KEY) return mockRegistry;
                return undefined;
            });

            const result = getRegistryContext();
            expect(mockGetContext).toHaveBeenCalledWith(REGISTRY_KEY);
            expect(result).toEqual(mockRegistry);
        });

        it('should throw an error when registry context is not in context', () => {
            mockGetContext.mockReturnValue(undefined);

            expect(() => getRegistryContext()).toThrow(
                'No registry provider was found in Svelte context. Did you forget to wrap your component with RegistryProvider?'
            );
        });
    });
}
