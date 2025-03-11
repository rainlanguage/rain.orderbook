import { render, screen, waitFor } from '@testing-library/svelte';
import StrategyPage from '../lib/components/deployment/StrategyPage.svelte';
import { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';
import { vi, describe, it, expect, beforeEach, type Mock } from 'vitest';

// Mock fetch
const mockFetch = vi.fn();
vi.stubGlobal('fetch', mockFetch);

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
			value: {
				name: 'Test Strategy',
				description: 'Test Description',
				short_description: 'Test Short Description'
			}
		};
		(DotrainOrderGui.getStrategyDetails as Mock).mockResolvedValueOnce(mockStrategyDetails);

		render(StrategyPage, {
			props: {
				dotrain: mockDotrain
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
			value: {
				name: 'Test Strategy',
				description: 'Test Description',
				short_description: 'Test Short Description'
			}
		};

		// Mock fetch response
		mockFetch.mockResolvedValueOnce({
			text: () => Promise.resolve(mockDotrain)
		});

		// Mock DotrainOrderGui methods
		(DotrainOrderGui.getStrategyDetails as Mock).mockResolvedValueOnce(mockStrategyDetails);

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

	it('displays error message when strategy details fail', async () => {
		const mockDotrain = 'mock dotrain content';

		// Mock fetch response
		mockFetch.mockResolvedValueOnce({
			text: () => Promise.resolve(mockDotrain)
		});

		// Mock DotrainOrderGui methods
		(DotrainOrderGui.getStrategyDetails as Mock).mockResolvedValueOnce({
			error: {
				msg: 'Failed to get strategy details'
			}
		});

		render(StrategyPage, {
			props: {
				strategyName: 'TestStrategy',
				dotrain: mockDotrain
			}
		});

		await waitFor(() => {
			expect(screen.getByText('Error getting strategy details')).toBeInTheDocument();
			expect(screen.getByText('Failed to get strategy details')).toBeInTheDocument();
		});
	});

	it('handles fetch failure', async () => {
		const mockError = new Error('Failed to fetch');

		(DotrainOrderGui.getStrategyDetails as Mock).mockResolvedValueOnce({
			value: {
				name: 'Test Strategy',
				description: 'https://example.com/description.md',
				short_description: 'Test Short Description'
			}
		});
		// Mock fetch to reject
		mockFetch.mockRejectedValueOnce(mockError);

		render(StrategyPage, {
			props: {
				strategyName: 'TestStrategy'
			}
		});

		await waitFor(() => {
			expect(screen.getByText('Error fetching markdown')).toBeInTheDocument();
			expect(screen.getByText('Failed to fetch markdown content')).toBeInTheDocument();
		});
	});

	it('renders markdown if description is a markdown url', async () => {
		const mockDotrain = 'mock dotrain content';
		const mockStrategyDetails = {
			value: {
				name: 'Test Strategy',
				description: 'https://example.com/description.md',
				short_description: 'Test Short Description'
			}
		};
		const mockMarkdownContent = '# Mock Markdown Content';

		// First fetch for dotrain
		mockFetch.mockResolvedValueOnce({
			ok: true,
			text: () => Promise.resolve(mockDotrain)
		});

		// Second fetch for markdown content
		mockFetch.mockResolvedValueOnce({
			ok: true,
			text: () => Promise.resolve(mockMarkdownContent)
		});

		(DotrainOrderGui.getStrategyDetails as Mock).mockResolvedValueOnce(mockStrategyDetails);

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
			expect(mockFetch).toHaveBeenCalledWith('https://example.com/description.md');
		});
	});

	it('falls back to plain text when markdown fetch fails', async () => {
		const mockDotrain = 'mock dotrain content';
		const mockStrategyDetails = {
			value: {
				name: 'Test Strategy',
				description: 'https://example.com/description.md',
				short_description: 'Test Short Description'
			}
		};

		mockFetch
			.mockResolvedValueOnce({
				ok: true,
				text: () => Promise.resolve(mockDotrain)
			})
			.mockResolvedValueOnce({
				ok: false,
				statusText: 'Not Found'
			});

		(DotrainOrderGui.getStrategyDetails as Mock).mockResolvedValueOnce(mockStrategyDetails);

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
