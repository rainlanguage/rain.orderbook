import { render } from '@testing-library/svelte';
import InputToken from '../lib/components/input/InputToken.svelte';


describe('InputToken', () => {
	it('renders with initial values', () => {
		const address = '0xc0D477556c25C9d67E1f57245C7453DA776B51cf';
		const decimals = 10;
		const { getByTestId } = render(InputToken, { props: { address, decimals } });

		const input = getByTestId('token-address-input').querySelector('input');
		expect(input?.value).toBe('0xc0D477556c25C9d67E1f57245C7453DA776B51cf');
		const decimalsInput = getByTestId('token-decimals-input').querySelector('input');
		expect(decimalsInput?.value).toBe('10');
	});
});
