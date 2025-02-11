import { render, screen } from '@testing-library/svelte';
import { describe, it, expect } from 'vitest';
import DeploymentSectionHeader from '../lib/components/deployment/DeploymentSectionHeader.svelte';
import type { ComponentProps } from 'svelte';

export type DeploymentSectionHeaderComponentProps = ComponentProps<DeploymentSectionHeader>;

describe('DeploymentSectionHeader', () => {
	const defaultProps: DeploymentSectionHeaderComponentProps = {
		title: 'Test Title',
		description: 'Test Description'
	};

	it('renders title and description', () => {
		render(DeploymentSectionHeader, defaultProps);

		expect(screen.getByText('Test Title')).toBeInTheDocument();
		expect(screen.getByText('Test Description')).toBeInTheDocument();
	});

	it('shows value when not open', () => {
		render(DeploymentSectionHeader, defaultProps);

		expect(screen.getByTestId('header-value')).toBeInTheDocument();
		expect(screen.getByText('Test Value')).toBeInTheDocument();
	});

	it('hides value when open', () => {
		render(DeploymentSectionHeader, {
			...defaultProps
		});

		expect(screen.queryByTestId('header-value')).not.toBeInTheDocument();
	});

	it('hides value when value is undefined', () => {
		render(DeploymentSectionHeader, {
			...defaultProps
		});

		expect(screen.queryByTestId('header-value')).not.toBeInTheDocument();
	});
});
