import type { SgErc20 } from '@rainlanguage/orderbook';

export function getTokenDisplayName(token: SgErc20): string {
	return token.symbol || token.name || 'Unknown Token';
}
