import { invalidateTanstackQueries } from '@rainlanguage/ui-components';
import { handleDepositOrWithdrawModal } from './modal';
import type { SgVault } from '@rainlanguage/orderbook';
import type { Hex } from 'viem';
import type { QueryClient } from '@tanstack/svelte-query';

interface VaultActionParams {
	vault: SgVault;
	action: 'deposit' | 'withdraw';
	chainId: number;
	rpcUrl: string;
	subgraphUrl: string;
	account: Hex;
	queryClient: QueryClient;
	vaultId: string;
}

export function handleVaultAction({
	vault,
	action,
	chainId,
	rpcUrl,
	subgraphUrl,
	account,
	queryClient,
	vaultId
}: VaultActionParams) {
	handleDepositOrWithdrawModal({
		open: true,
		args: {
			vault,
			onDepositOrWithdraw: () => {
				invalidateTanstackQueries(queryClient, [vaultId]);
			},
			action,
			chainId,
			rpcUrl,
			subgraphUrl,
			account
		}
	});
}
