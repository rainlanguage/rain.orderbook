import type { SgVault } from '@rainlanguage/orderbook';
import type { Hex } from 'viem';
import { handleDepositModal } from '$lib/services/modal';

interface handleVaultDepositParams {
	vault: SgVault;
	chainId: number;
	rpcUrl: string;
	subgraphUrl: string;
	account: Hex;
}

export function handleVaultDeposit({
	vault,
	chainId,
	rpcUrl,
	subgraphUrl,
	account
}: handleVaultDepositParams) {
	handleDepositModal({
		open: true,
		args: {
			vault,
			chainId,
			rpcUrl,
			subgraphUrl,
			account
		}
	});
}
