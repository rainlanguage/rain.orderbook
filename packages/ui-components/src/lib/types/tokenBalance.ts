import type { AccountBalance } from '@rainlanguage/orderbook';

export interface TokenBalance {
	value: AccountBalance;
	loading: boolean;
	error: string;
}
