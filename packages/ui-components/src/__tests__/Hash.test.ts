import { getByTestId, render } from '@testing-library/svelte';
import Hash from '../lib/components/Hash.svelte';
import { describe, it, expect } from 'vitest';
import userEvent from '@testing-library/user-event';
import type { ComponentProps } from 'svelte';
import truncateEthAddress from 'truncate-eth-address';
type HashComponentProps = ComponentProps<Hash>;

const mockProps: HashComponentProps = {
	value: 'abcdef1234567890',
	type: 1, // HashType.Wallet
	shorten: true
};

describe('Hash Component', () => {
	it('renders with shortened hash display', () => {
		const { getByText } = render(Hash, {
			props: mockProps
		});
		expect(getByText(truncateEthAddress(mockProps.value))).toBeInTheDocument();
	});

	it('renders full hash when shorten is false', () => {
		const { getByText } = render(Hash, {
			props: {
				...mockProps,
				shorten: false
			}
		});

		expect(getByText(mockProps.value)).toBeInTheDocument();
	});

	it('copies hash to clipboard and shows copied message', async () => {
		const user = userEvent.setup();

		const { getByRole, findByText } = render(Hash, {
			props: {
				...mockProps,
				copyOnClick: true
			}
		});

		const button = getByRole('button');
		await user.click(button);
		expect(await findByText('Copied to clipboard')).toBeInTheDocument();
		const clipboardText = await navigator.clipboard.readText();
		expect(clipboardText).toBe(mockProps.value);
	});

	it('does not copy to clipboard if copyOnClick is false', async () => {
		const user = userEvent.setup();

		const { getByRole } = render(Hash, {
			props: {
				...mockProps,
				copyOnClick: false
			}
		});

		const button = getByRole('button');
		await user.click(button);
		const clipboardText = await navigator.clipboard.readText();
		expect(clipboardText).toBe('');
	});

	it('renders with external link when linkType is provided', () => {
		const { getByTestId } = render(Hash, {
			props: {
				...mockProps,
				linkType: 'address',
				chainId: 1
			}
		});

		const link = getByTestId('external-link');
		expect(link).toHaveAttribute('href', `https://etherscan.io/address/${mockProps.value}`);
	});
});
