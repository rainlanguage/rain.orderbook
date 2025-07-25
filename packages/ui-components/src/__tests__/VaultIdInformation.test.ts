import { render, screen } from '@testing-library/svelte';
import { describe, it, expect } from 'vitest';
import type { ComponentProps } from 'svelte';
import VaultIdInformation from '$lib/components/deployment/VaultIdInformation.svelte';
import type { AccountBalance } from '@rainlanguage/orderbook';

export type VaultIdInformationComponentProps = ComponentProps<VaultIdInformation>;

describe('VaultIdInformation', () => {
	const defaultProps: VaultIdInformationComponentProps = {
		title: 'Test Title',
		description: 'Test Description',
		tokenBalance: {
			value: {
				balance: 1000n,
				formattedBalance: '100'
			} as AccountBalance,
			loading: false,
			error: ''
		}
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
				value: {
					balance: 0n,
					formattedBalance: '0'
				} as AccountBalance,
				loading: true,
				error: ''
			}
		};

		render(VaultIdInformation, loadingProps);

		expect(screen.queryByText(/Balance:/)).not.toBeInTheDocument();
	});

	it('shows error message when tokenBalance has error', () => {
		const errorProps: VaultIdInformationComponentProps = {
			...defaultProps,
			tokenBalance: {
				value: {
					balance: 0n,
					formattedBalance: '0'
				} as AccountBalance,
				loading: false,
				error: 'Failed to fetch balance'
			}
		};

		render(VaultIdInformation, errorProps);

		expect(screen.getByText('Failed to fetch balance')).toBeInTheDocument();
		expect(screen.queryByText(/Balance:/)).not.toBeInTheDocument();
	});
});
