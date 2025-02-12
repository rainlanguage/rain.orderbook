import { render, screen, waitFor } from '@testing-library/svelte';
import StrategySection from '../lib/components/deployment/StrategySection.svelte';
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

vi.mock('svelte-markdown', async () => {
	const mockSvelteMarkdown = (await import('../lib/__mocks__/MockComponent.svelte')).default;
	return { default: mockSvelteMarkdown };
});

describe('StrategySection', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('renders strategy details successfully with rawDotrain', async () => {
		const mockDotrain = 'mock dotrain content';
		const mockStrategyDetails = {
			name: 'Test Strategy',
			description: 'Test Description'
		};
		vi.mocked(DotrainOrderGui.getStrategyDetails).mockResolvedValueOnce(mockStrategyDetails);

		render(StrategySection, {
			props: {
				rawDotrain: mockDotrain
			}
		});

		await waitFor(() => {
			expect(screen.getByText('Test Strategy')).toBeInTheDocument();
			expect(screen.getByText('Test Description')).toBeInTheDocument();
		});
	});

	it('renders strategy details successfully from fetch', async () => {
		const mockDotrain = 'mock dotrain content';
		const mockStrategyDetails = {
			name: 'Test Strategy',
			description: 'Test Description'
		};

		// Mock fetch response
		mockFetch.mockResolvedValueOnce({
			text: () => Promise.resolve(mockDotrain)
		});

		// Mock DotrainOrderGui methods
		vi.mocked(DotrainOrderGui.getStrategyDetails).mockResolvedValueOnce(mockStrategyDetails);

		render(StrategySection, {
			props: {
				strategyUrl: 'http://example.com/strategy',
				strategyName: 'TestStrategy'
			}
		});

		await waitFor(() => {
			expect(screen.getByText('Test Strategy')).toBeInTheDocument();
			expect(screen.getByText('Test Description')).toBeInTheDocument();
		});
	});

	it('displays error message when strategy details fail', async () => {
		const mockDotrain = 'mock dotrain content';
		const mockError = new Error('Failed to get strategy details');

		// Mock fetch response
		mockFetch.mockResolvedValueOnce({
			text: () => Promise.resolve(mockDotrain)
		});

		// Mock DotrainOrderGui methods
		vi.mocked(DotrainOrderGui.getStrategyDetails).mockRejectedValueOnce(mockError);

		render(StrategySection, {
			props: {
				strategyUrl: 'http://example.com/strategy',
				strategyName: 'TestStrategy'
			}
		});

		await waitFor(() => {
			expect(screen.getByText('Error getting strategy details')).toBeInTheDocument();
			expect(screen.getByText('Failed to get strategy details')).toBeInTheDocument();
		});
	});

	it('handles fetch failure', async () => {
		const mockError = new Error('Failed to fetch');

		// Mock fetch to reject
		mockFetch.mockRejectedValueOnce(mockError);

		render(StrategySection, {
			props: {
				strategyUrl: 'http://example.com/strategy',
				strategyName: 'TestStrategy'
			}
		});

		await waitFor(() => {
			expect(screen.getByText('Error fetching strategy')).toBeInTheDocument();
			expect(screen.getByText('Failed to fetch')).toBeInTheDocument();
		});
	});

	it('renders markdown if description is a markdown url', async () => {
		const mockDotrain = 'mock dotrain content';
		const mockStrategyDetails = {
			name: 'Test Strategy',
			description: 'https://example.com/description.md'
		};
		const mockMarkdownContent = 'mock markdown content';

		mockFetch
			.mockResolvedValueOnce({
				ok: true,
				text: () => Promise.resolve(mockDotrain)
			})

			.mockResolvedValueOnce({
				ok: true,
				text: () => Promise.resolve(mockMarkdownContent)
			});

		vi.mocked(DotrainOrderGui.getStrategyDetails).mockResolvedValueOnce(mockStrategyDetails);

		render(StrategySection, {
			props: {
				strategyUrl: 'http://example.com/strategy',
				strategyName: 'TestStrategy'
			}
		});

		await waitFor(() => {
			expect(screen.getByText('Test Strategy')).toBeInTheDocument();
			expect(screen.getByTestId('markdown-content')).toBeInTheDocument();
			expect(mockFetch).toHaveBeenCalledWith('https://example.com/description.md');
		});
	});
});
