import { render } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import DotrainRegistryProvider from '../lib/providers/dotrainRegistry/DotrainRegistryProvider.svelte';
import { setDotrainRegistryContext } from '../lib/providers/dotrainRegistry/context';

vi.mock('../lib/providers/dotrainRegistry/context', () => ({
	setDotrainRegistryContext: vi.fn()
}));

describe('DotrainRegistryProvider', () => {
	const mockSetContext = vi.mocked(setDotrainRegistryContext);

	beforeEach(() => {
		mockSetContext.mockReset();
	});

	it('sets the registry context with provided props', () => {
		const registry = {} as never;
		const manager = {} as never;
		render(DotrainRegistryProvider, { props: { registry, manager, error: undefined } });

		expect(mockSetContext).toHaveBeenCalledWith({
			registry,
			manager,
			error: undefined
		});
	});
});
