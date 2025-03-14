import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import InvalidStrategiesSection from '../lib/components/deployment/InvalidStrategiesSection.svelte';
import type { InvalidStrategyDetail } from '$lib/types/strategy';

describe('InvalidStrategiesSection', () => {
	const mockInvalidStrategies: InvalidStrategyDetail[] = [
		{
			name: 'Strategy 1',
			error: 'Invalid configuration'
		},
		{
			name: 'Strategy 2',
			error: 'Missing required field'
		}
	];

	it('displays multiple invalid strategies with their errors', () => {
		render(InvalidStrategiesSection, { props: { strategiesWithErrors: mockInvalidStrategies } });

		expect(screen.getByText('Strategy 1')).toBeInTheDocument();
		expect(screen.getByText('Strategy 2')).toBeInTheDocument();
		expect(screen.getByText('Invalid configuration')).toBeInTheDocument();
		expect(screen.getByText('Missing required field')).toBeInTheDocument();
	});
});
