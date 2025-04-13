import { describe, beforeEach, it, expect, vi, type Mock } from 'vitest';
import { render, screen, waitFor } from '@testing-library/svelte';
import Page from './+page.svelte';
import { readable } from 'svelte/store';
import {
	useRegistry,
	type ValidStrategyDetail,
	type InvalidStrategyDetail
} from '@rainlanguage/ui-components';

const { mockPageStore } = await vi.hoisted(
	() => import('$lib/__mocks__/stores')
);

vi.mock('$app/stores', async (importOriginal) => {
	return {
		...((await importOriginal()) as object),
		page: mockPageStore
	};
});

const mockValidStrategy1: ValidStrategyDetail = {
	details: {
		name: 'Strategy One',
		description: 'This is the first valid strategy.',
		short_description: 'Valid 1'
	},
	name: 'strategy1.dotrain',
	dotrain: ';;'
};

const mockRegistry = vi.fn();
const mockIsCustomRegistry = vi.fn();

const mockValidStrategy2: ValidStrategyDetail = {
	details: {
		name: 'Strategy Two',
		description: 'Another valid strategy.',
		short_description: 'Valid 2'
	},
	name: 'strategy2.dotrain',
	dotrain: ';;'
};

const mockInvalidStrategy1: InvalidStrategyDetail = {
	name: 'invalidStrategy.dotrain',
	error: 'Syntax error on line 1'
};

vi.mock('@rainlanguage/ui-components', async (importOriginal) => {
	return {
		...((await importOriginal()) as object),
		useRegistry: vi.fn()
	};
});

const mockGetCurrentRegistry = vi.fn().mockReturnValue(readable({}));

describe('Page Component', () => {
	const mockDotrains = [
		mockValidStrategy1.dotrain,
		mockValidStrategy2.dotrain,
		mockInvalidStrategy1.name
	];

	const mockValidated = {
		validStrategies: [mockValidStrategy1, mockValidStrategy2],
		invalidStrategies: [mockInvalidStrategy1]
	};

	beforeEach(() => {
		vi.resetAllMocks();
		(useRegistry as Mock).mockReturnValue(
			readable({
				getCurrentRegistry: mockGetCurrentRegistry,
				isCustomRegistry: mockIsCustomRegistry,
				subscribe: vi.fn()
			})
		);
		mockIsCustomRegistry.mockReturnValue(true);
		mockPageStore.reset();
	});

	it('should display error message when fetching strategies fails', async () => {
		mockPageStore.mockSetSubscribeValue({
			data: {
				error: 'Failed to fetch registry dotrains'
			}
		});

		render(Page, {
			context: new Map([['$$_registry', mockRegistry]])
		});

		await waitFor(() => {
			expect(screen.getByText(/Failed to load strategies:/i)).toBeInTheDocument();
			expect(screen.getByText('Error: Failed to fetch registry dotrains')).toBeInTheDocument();
		});
	});

	it('should display error message when validating strategies fails', async () => {
		mockPageStore.mockSetSubscribeValue({
			data: {
				error: 'Failed to validate strategies'
			}
		});

		render(Page, {
			context: new Map([['$$_registry', mockRegistry]])
		});

		await waitFor(() => {
			expect(screen.getByText(/Failed to load strategies:/i)).toBeInTheDocument();
			expect(screen.getByText('Error: Failed to validate strategies')).toBeInTheDocument();
		});
	});

	it('should display no strategies found when no strategies are available', async () => {
		mockPageStore.mockSetSubscribeValue({
			data: {
        error: null,
				validStrategies: [],
				invalidStrategies: []
			}
		});

		render(Page, {
			context: new Map([['$$_registry', mockRegistry]])
		});

		await waitFor(() => {
			expect(screen.getByText('No strategies found')).toBeInTheDocument();
		});
	});

	it('should display valid strategies when they are available', async () => {
		mockPageStore.mockSetSubscribeValue({
			data: {
				validStrategies: mockValidated.validStrategies,
				invalidStrategies: []
			}
		});

		render(Page, {
			context: new Map([['$$_registry', mockRegistry]])
		});

		await waitFor(() => {
			expect(screen.getByTestId('valid-strategies')).toBeInTheDocument();
		});
	});

	it('should display invalid strategies when they are available', async () => {
		mockPageStore.mockSetSubscribeValue({
			data: {
				validStrategies: [],
				invalidStrategies: mockValidated.invalidStrategies
			}
		});

		render(Page, {
			context: new Map([['$$_registry', mockRegistry]])
		});

		await waitFor(() => {
			expect(screen.getByTestId('invalid-strategies')).toBeInTheDocument();
		});
	});

	it('should display valid and invalid strategies when both are available', async () => {
		mockPageStore.mockSetSubscribeValue({
			data: {
				validStrategies: mockValidated.validStrategies,
				invalidStrategies: mockValidated.invalidStrategies
			}
		});

		render(Page, {
			context: new Map([['$$_registry', mockRegistry]])
		});

		await waitFor(() => {
			expect(screen.getByTestId('valid-strategies')).toBeInTheDocument();
			expect(screen.getByTestId('invalid-strategies')).toBeInTheDocument();
		});
	});
});
