import { describe, beforeEach, it, expect, vi, type Mock } from 'vitest';
import { render, screen, waitFor } from '@testing-library/svelte';
import Page from './+page.svelte';
import { readable } from 'svelte/store';
import {
	useRegistry,
	fetchRegistryDotrains,
	validateStrategies,
	type ValidStrategyDetail,
	type InvalidStrategyDetail
} from '@rainlanguage/ui-components';

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
		useRegistry: vi.fn(),
		validateStrategies: vi.fn(),
		fetchRegistryDotrains: vi.fn()
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
	});

	it('should display error message when fetching strategies fails', async () => {
		const errorMessage = 'Failed to fetch registry dotrains';
		(fetchRegistryDotrains as Mock).mockRejectedValue(new Error(errorMessage));

		render(Page, {
			context: new Map([['$$_registry', mockRegistry]])
		});

		await waitFor(() => {
			expect(screen.getByText(/Failed to load strategies:/i)).toBeInTheDocument();
			expect(screen.getByText('Error: ' + errorMessage)).toBeInTheDocument();
		});
	});

	it('should display error message when validating strategies fails', async () => {
		(fetchRegistryDotrains as Mock).mockResolvedValue(mockDotrains);
		const errorMessage = 'Failed to validate strategies';
		(validateStrategies as Mock).mockRejectedValue(new Error(errorMessage));

		render(Page, {
			context: new Map([['$$_registry', mockRegistry]])
		});

		await waitFor(() => {
			expect(screen.getByText(/Failed to load strategies:/i)).toBeInTheDocument();
			expect(screen.getByText('Error: ' + errorMessage)).toBeInTheDocument();
		});
	});

	it('should display no strategies found when no strategies are available', async () => {
		(fetchRegistryDotrains as Mock).mockResolvedValue([]);
		(validateStrategies as Mock).mockResolvedValue({
			validStrategies: [],
			invalidStrategies: []
		});

		render(Page, {
			context: new Map([['$$_registry', mockRegistry]])
		});

		await waitFor(() => {
			expect(screen.getByText('No strategies found')).toBeInTheDocument();
		});
	});

	it('should display valid strategies when they are available', async () => {
		(fetchRegistryDotrains as Mock).mockResolvedValue(mockDotrains);
		(validateStrategies as Mock).mockResolvedValue({
			validStrategies: mockValidated.validStrategies,
			invalidStrategies: []
		});

		render(Page, {
			context: new Map([['$$_registry', mockRegistry]])
		});

		await waitFor(() => {
			expect(screen.getByTestId('valid-strategies')).toBeInTheDocument();
		});
	});

	it('should display invalid strategies when they are available', async () => {
		(fetchRegistryDotrains as Mock).mockResolvedValue(mockDotrains);
		vi.mocked(validateStrategies).mockResolvedValue({
			validStrategies: [],
			invalidStrategies: mockValidated.invalidStrategies
		});

		render(Page, {
			context: new Map([['$$_registry', mockRegistry]])
		});

		await waitFor(() => {
			expect(screen.getByTestId('invalid-strategies')).toBeInTheDocument();
		});
	});

	it('should display valid and invalid strategies when both are available', async () => {
		(fetchRegistryDotrains as Mock).mockResolvedValue(mockDotrains);
		(validateStrategies as Mock).mockResolvedValue(mockValidated);

		render(Page, {
			context: new Map([['$$_registry', mockRegistry]])
		});

		await waitFor(() => {
			expect(screen.getByTestId('valid-strategies')).toBeInTheDocument();
			expect(screen.getByTestId('invalid-strategies')).toBeInTheDocument();
		});
	});
});
