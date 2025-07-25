import { render, screen } from '@testing-library/svelte';
import { describe, it, expect } from 'vitest';
import TokenBalance from '../lib/components/deployment/TokenBalance.svelte';
import type { ComponentProps } from 'svelte';
import type { TokenBalance as TokenBalanceType } from '$lib/types/tokenBalance';
import type { AccountBalance } from '@rainlanguage/orderbook';

type TokenBalanceComponentProps = ComponentProps<TokenBalance>;

describe('TokenBalance', () => {
	const createMockTokenBalance = (
		balance: bigint = BigInt(0),
		formattedBalance: string = '0',
		loading: boolean = false,
		error: string = ''
	): TokenBalanceType => ({
		value: { balance, formattedBalance } as AccountBalance,
		loading,
		error
	});

	const defaultProps: TokenBalanceComponentProps = {
		tokenBalance: createMockTokenBalance()
	};

	it('renders loading state correctly', () => {
		const props: TokenBalanceComponentProps = {
			...defaultProps,
			tokenBalance: createMockTokenBalance(BigInt(0), '0', true, '')
		};

		render(TokenBalance, { props });

		expect(screen.getByRole('status')).toBeInTheDocument();
	});

	it('renders balance when balance is non-zero', () => {
		const props: TokenBalanceComponentProps = {
			...defaultProps,
			tokenBalance: createMockTokenBalance(BigInt(1000), '1000', false, '')
		};

		render(TokenBalance, { props });

		expect(screen.getByText('Balance: 1000')).toBeInTheDocument();
	});

	it('renders error state correctly', () => {
		const props: TokenBalanceComponentProps = {
			...defaultProps,
			tokenBalance: createMockTokenBalance(BigInt(0), '0', false, 'Failed to fetch balance')
		};

		render(TokenBalance, { props });

		expect(screen.getByText('Failed to fetch balance')).toBeInTheDocument();
	});

	it('applies correct color classes for balance', () => {
		const { container } = render(TokenBalance, {
			props: {
				...defaultProps,
				tokenBalance: createMockTokenBalance(BigInt(1000), '1,000.00', false, '')
			}
		});

		expect(container.querySelector('.text-gray-600')).toBeInTheDocument();
	});

	it('uses red color for error state', () => {
		const { container } = render(TokenBalance, {
			props: {
				...defaultProps,
				tokenBalance: createMockTokenBalance(BigInt(0), '0', false, 'Error message')
			}
		});

		expect(container.querySelector('.text-red-600')).toBeInTheDocument();
	});

	it('prioritizes error display over balance when both exist', () => {
		const props: TokenBalanceComponentProps = {
			...defaultProps,
			tokenBalance: createMockTokenBalance(BigInt(1000), '1000', false, 'Network error')
		};

		render(TokenBalance, { props });

		expect(screen.getByText('Network error')).toBeInTheDocument();
		expect(screen.queryByText('Balance: 1000')).not.toBeInTheDocument();
	});
});
