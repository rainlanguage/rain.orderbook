import { render, screen, waitFor } from '@testing-library/svelte';
import StrategyPage from '../lib/components/deployment/StrategyPage.svelte';
import { DotrainOrderGui, type NameAndDescriptionCfg } from '@rainlanguage/orderbook/js_api';
import { vi, describe, it, expect, beforeEach } from 'vitest';
import { type WasmEncodedResult } from '@rainlanguage/orderbook/js_api';

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
			description: 'Test Description',
			short_description: 'Test Short Description'
		} as unknown as WasmEncodedResult<NameAndDescriptionCfg>;
		vi.mocked(DotrainOrderGui.getStrategyDetails).mockResolvedValueOnce(mockStrategyDetails);

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
			name: 'Test Strategy',
			description: 'Test Description',
			short_description: 'Test Short Description'
		} as unknown as WasmEncodedResult<NameAndDescriptionCfg>;

		// Mock fetch response
		mockFetch.mockResolvedValueOnce({
			text: () => Promise.resolve(mockDotrain)
		});

		// Mock DotrainOrderGui methods
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

	it('displays error message when strategy details fail', async () => {
		const mockDotrain = 'mock dotrain content';
		const mockError = new Error('Failed to get strategy details');

		// Mock fetch response
		mockFetch.mockResolvedValueOnce({
			text: () => Promise.resolve(mockDotrain)
		});

		// Mock DotrainOrderGui methods
		vi.mocked(DotrainOrderGui.getStrategyDetails).mockRejectedValueOnce(mockError);

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

		// Mock fetch to reject
		mockFetch.mockRejectedValueOnce(mockError);

		render(StrategyPage, {
			props: {
				strategyName: 'TestStrategy'
			}
		});

		await waitFor(() => {
			expect(screen.getByText('Error getting strategy details')).toBeInTheDocument();
			expect(
				screen.getByText("Cannot read properties of undefined (reading 'description')")
			).toBeInTheDocument();
		});
	});

	it('renders markdown if description is a markdown url', async () => {
		const mockDotrain = 'mock dotrain content';
		const mockStrategyDetails = {
			name: 'Test Strategy',
			description: 'https://example.com/description.md',
			short_description: 'Test Short Description'
		} as unknown as WasmEncodedResult<NameAndDescriptionCfg>;
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
			expect(mockFetch).toHaveBeenCalledWith('https://example.com/description.md');
		});
	});

	it('falls back to plain text when markdown fetch fails', async () => {
		const mockDotrain = 'mock dotrain content';
		const mockStrategyDetails = {
			name: 'Test Strategy',
			description: 'https://example.com/description.md',
			short_description: 'Test Short Description'
		} as unknown as WasmEncodedResult<NameAndDescriptionCfg>;

		mockFetch
			.mockResolvedValueOnce({
				ok: true,
				text: () => Promise.resolve(mockDotrain)
			})
			.mockResolvedValueOnce({
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
