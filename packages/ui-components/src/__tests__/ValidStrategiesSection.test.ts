import { render, screen } from '@testing-library/svelte';
import ValidStrategiesSection from '../lib/components/deployment/ValidStrategiesSection.svelte';
import type { StrategyDetail } from '$lib/types/strategy';

describe('ValidStrategiesSection', () => {
	const mockValidStrategies: StrategyDetail[] = [
		{
			dotrain: '',
			name: 'strategy1',
			details: {
				name: 'Strategy 1',
				description: 'Test strategy 1',
				short_description: 'Short description 1'
			}
		},
		{
			dotrain: '',
			name: 'strategy2',
			details: {
				name: 'Strategy 2',
				description: 'Test strategy 2',
				short_description: 'Short description 2'
			}
		}
	];

	it('should render nothing when validStrategies is empty', () => {
		render(ValidStrategiesSection, { props: { validStrategies: [] } });
		const container = screen.queryByTestId('valid-strategies');
		expect(container).not.toBeInTheDocument();
	});

	it('should render correct number of StrategyShortTile components', () => {
		render(ValidStrategiesSection, { props: { validStrategies: mockValidStrategies } });
		const strategyTiles = screen.getAllByTestId('strategy-short-tile');
		expect(strategyTiles).toHaveLength(mockValidStrategies.length);
	});
});
