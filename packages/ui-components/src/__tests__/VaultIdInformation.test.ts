import { render, screen } from '@testing-library/svelte';
import { describe, it, expect } from 'vitest';
import type { ComponentProps } from 'svelte';
import VaultIdInformation from '$lib/components/deployment/VaultIdInformation.svelte';

export type VaultIdInformationComponentProps = ComponentProps<VaultIdInformation>;

describe('VaultIdInformation', () => {
	const defaultProps: VaultIdInformationComponentProps = {
		title: 'Test Title',
		description: 'Test Description',
		tokenBalance: {
			balance: 1000n,
			loading: false,
			error: ''
		},
		decimals: 1
	};

	it('renders title, description, and token balance', () => {
		render(VaultIdInformation, defaultProps);

		expect(screen.getByText('Test Title')).toBeInTheDocument();
		expect(screen.getByText('Test Description')).toBeInTheDocument();
		expect(screen.getByText('Balance: 100')).toBeInTheDocument();
	});

	it('shows loading state when tokenBalance is loading', () => {
		const loadingProps: VaultIdInformationComponentProps = {
			...defaultProps,
			tokenBalance: {
				balance: null,
				loading: true,
				error: ''
			}
		};

		render(VaultIdInformation, loadingProps);

		expect(screen.getByText('Loading balance...')).toBeInTheDocument();
		expect(screen.queryByText(/Balance:/)).not.toBeInTheDocument();
	});

	it('shows error message when tokenBalance has error', () => {
		const errorProps: VaultIdInformationComponentProps = {
			...defaultProps,
			tokenBalance: {
				balance: null,
				loading: false,
				error: 'Failed to fetch balance'
			}
		};

		render(VaultIdInformation, errorProps);

		expect(screen.getByText('Failed to fetch balance')).toBeInTheDocument();
		expect(screen.queryByText(/Balance:/)).not.toBeInTheDocument();
		expect(screen.queryByText('Loading balance...')).not.toBeInTheDocument();
	});

	it('does not show balance when balance is null', () => {
		const nullBalanceProps: VaultIdInformationComponentProps = {
			...defaultProps,
			tokenBalance: {
				balance: null,
				loading: false,
				error: ''
			}
		};

		render(VaultIdInformation, nullBalanceProps);

		expect(screen.queryByText(/Balance:/)).not.toBeInTheDocument();
		expect(screen.queryByText('Loading balance...')).not.toBeInTheDocument();
	});

	it('does not show balance when decimals is undefined', () => {
		const undefinedDecimalsProps: VaultIdInformationComponentProps = {
			...defaultProps,
			decimals: undefined
		};

		render(VaultIdInformation, undefinedDecimalsProps);

		expect(screen.queryByText(/Balance:/)).not.toBeInTheDocument();
	});

	it('formats balance correctly with different decimal places', () => {
		const props18Decimals: VaultIdInformationComponentProps = {
			...defaultProps,
			tokenBalance: {
				balance: 1000000000000000000n, // 1 token with 18 decimals
				loading: false,
				error: ''
			},
			decimals: 18
		};

		render(VaultIdInformation, props18Decimals);

		expect(screen.getByText('Balance: 1')).toBeInTheDocument();
	});

	it('formats balance correctly with 6 decimal places', () => {
		const props6Decimals: VaultIdInformationComponentProps = {
			...defaultProps,
			tokenBalance: {
				balance: 1000000n, // 1 token with 6 decimals
				loading: false,
				error: ''
			},
			decimals: 6
		};

		render(VaultIdInformation, props6Decimals);

		expect(screen.getByText('Balance: 1')).toBeInTheDocument();
	});

	it('formats fractional balance correctly', () => {
		const fractionalProps: VaultIdInformationComponentProps = {
			...defaultProps,
			tokenBalance: {
				balance: 1500000000000000000n, // 1.5 tokens with 18 decimals
				loading: false,
				error: ''
			},
			decimals: 18
		};

		render(VaultIdInformation, fractionalProps);

		expect(screen.getByText('Balance: 1.5')).toBeInTheDocument();
	});
});
