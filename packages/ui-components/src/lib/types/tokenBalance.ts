import type { RaindexAmount } from '@rainlanguage/orderbook';

export interface TokenBalance {
	value: RaindexAmount;
	loading: boolean;
	error: string;
}
