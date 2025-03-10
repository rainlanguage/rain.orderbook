import { render, screen, waitFor } from '@testing-library/svelte';
import StrategyPage from '../lib/components/deployment/StrategyPage.svelte';
import { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';
import { vi, describe, it, expect, beforeEach } from 'vitest';

// Mock fetch
const mockFetch = vi.fn();
vi.stubGlobal('fetch', mockFetch);

// Mock DotrainOrderGui
vi.mock('@rainlanguage/orderbook/js_api', () => ({
	DotrainOrderGui: {
		getStrategyDetails: vi.fn(),
		getDeploymentDetails: vi.fn()
	}
}));

describe('StrategyPage', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		vi.resetAllMocks();
	});

	it.only('renders strategy details successfully with rawDotrain', async () => {
		const mockDotrain = 'mock dotrain content';
		const mockStrategyDetails = {
			name: 'Test Strategy',
			description: 'Test Description',
			short_description: 'Test Short Description'
		};
		const strategyPromise = Promise.resolve(mockStrategyDetails);
		vi.mocked(DotrainOrderGui.getStrategyDetails).mockReturnValue(strategyPromise);

		render(StrategyPage, {
			props: {
				dotrain: mockDotrain
			}
		});
		await strategyPromise;

		await waitFor(() => {
			expect(screen.getByText('Test Strategy')).toBeInTheDocument();
			expect(screen.getByText('Test Description')).toBeInTheDocument();
		});
	});

	it.only('renders strategy details successfully from fetch', async () => {
		const mockDotrain = 'mock dotrain content';
		const mockStrategyDetails = {
			name: 'Test Strategy',
			description: 'Test Description',
			short_description: 'Test Short Description'
		};

		mockFetch.mockResolvedValueOnce({
			text: () => Promise.resolve(mockDotrain)
		});

		vi.mocked(DotrainOrderGui.getStrategyDetails).mockResolvedValueOnce(mockStrategyDetails);

		render(StrategyPage, {
			props: {
				strategyName: 'TestStrategy',
				dotrain: mockDotrain
			}
		});

		await waitFor(() => {
			expect(screen.getByText('Test Strategy')).toBeInTheDocument();
			expect(screen.getByText('Test Description')).toBeInTheDocument();
		});
	});

	it.only('displays error message when strategy details fail', async () => {
		const mockDotrain = 'mock dotrain content';

		vi.mocked(DotrainOrderGui.getStrategyDetails).mockRejectedValueOnce(
			new Error('Error: Failed to get strategy details')
		);

		render(StrategyPage, {
			props: {
				strategyName: 'TestStrategy',
				dotrain: mockDotrain
			}
		});

		await waitFor(() => {
			expect(screen.getByText('Error: Failed to get strategy details')).toBeInTheDocument();
		});
	});

	it.only('handles markdown fetch failure', async () => {
		const mockDotrain = 'mock dotrain content';
		const mockStrategyDetails = {
			name: 'Test Strategy',
			description: 'https://example.com/description.md',
			short_description: 'Test Short Description'
		};

		mockFetch.mockRejectedValueOnce(new Error('Failed to fetch'));

		vi.mocked(DotrainOrderGui.getStrategyDetails).mockResolvedValueOnce(mockStrategyDetails);

		render(StrategyPage, {
			props: {
				strategyName: 'TestStrategy',
				dotrain: mockDotrain
			}
		});

		await waitFor(() => {
			expect(screen.getByText('https://example.com/description.md')).toBeInTheDocument();
		});
	});

	it.only('renders markdown if description is a markdown url', async () => {
		const mockDotrain = 'mock dotrain content';
		const mockStrategyDetails = {
			name: 'Test Strategy',
			description: 'https://example.com/description.md',
			short_description: 'Test Short Description'
		};

		mockFetch.mockResolvedValueOnce({
			ok: true,
			text: () => Promise.resolve('mock markdown content')
		});

		vi.mocked(DotrainOrderGui.getStrategyDetails).mockResolvedValueOnce(mockStrategyDetails);

		render(StrategyPage, {
			props: {
				strategyName: 'TestStrategy',
				dotrain: mockDotrain
			}
		});

		await waitFor(() => {
			expect(screen.getByText('Test Strategy')).toBeInTheDocument();
			expect(screen.getByTestId('markdown-content')).toBeInTheDocument();
			expect(mockFetch).toHaveBeenCalledWith('https://example.com/description.md');
		});
	});

	it.only('falls back to plain text when markdown fetch fails', async () => {
		const mockDotrain = 'mock dotrain content';
		const mockStrategyDetails = {
			name: 'Test Strategy',
			description: 'https://example.com/description.md',
			short_description: 'Test Short Description'
		};

		mockFetch.mockResolvedValueOnce({
			ok: false,
			statusText: 'Not Found'
		});

		vi.mocked(DotrainOrderGui.getStrategyDetails).mockResolvedValueOnce(mockStrategyDetails);

		render(StrategyPage, {
			props: {
				strategyName: 'TestStrategy',
				dotrain: mockDotrain
			}
		});

		await waitFor(() => {
			expect(screen.getByText('Test Strategy')).toBeInTheDocument();
			expect(screen.getByTestId('plain-description')).toHaveTextContent(
				'https://example.com/description.md'
			);
		});
	});
});
