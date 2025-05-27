import type { SgVault } from '@rainlanguage/orderbook';
import { invalidateTanstackQueries } from '@rainlanguage/ui-components';
import type { QueryClient } from '@tanstack/svelte-query';
import type { Hex } from 'viem';
import { handleDepositModal, handleWithdrawModal } from '$lib/services/modal';

interface HandleVaultActionParams {
	vault: SgVault;
	action: 'deposit' | 'withdraw';
	queryClient: QueryClient;
	queryKey: string;
	chainId: number;
	rpcUrl: string;
	subgraphUrl: string;
	account: Hex;
}

export function handleVaultAction({
	vault,
	action,
	queryClient,
	queryKey,
	chainId,
	rpcUrl,
	subgraphUrl,
	account
}: HandleVaultActionParams) {
	const modalHandler = action === 'deposit' ? handleDepositModal : handleWithdrawModal;
	modalHandler({
		open: true,
		args: {
			vault,
			onSuccess: () => {
				invalidateTanstackQueries(queryClient, [queryKey]);
			},
			chainId,
			rpcUrl,
			subgraphUrl,
			account
		}
	});
}