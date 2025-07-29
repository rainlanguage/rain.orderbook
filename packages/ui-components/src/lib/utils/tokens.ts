import type { RaindexVaultToken } from '@rainlanguage/orderbook';

export function getTokenDisplayName(token: RaindexVaultToken): string {
	return token.symbol || token.name || 'Unknown Token';
}
