import type { DeploymentTransactionArgs } from "@rainlanguage/orderbook";
import type { DeploymentArgs, TransactionConfirmationProps, TransactionManager } from "@rainlanguage/ui-components";
import type { Hex } from "viem";
import { sendTransaction } from "viem/actions";
import { handleTransactionConfirmationModal } from "./modal";

type HandleAddOrderDependencies = {
    args: DeploymentTransactionArgs;
    handleTransactionConfirmationModal: (props: TransactionConfirmationProps) => void;
	errToast: (message: string) => void;
	manager: TransactionManager;
	network: string;
	orderbookAddress: Hex;
}

export const handleAddOrder = async(deps: HandleAddOrderDependencies) => {
    const { approvals, deploymentCalldata, chainId, network, subgraphUrl } = deps.args;
    		for (const approval of approvals) {
                await handleTransactionConfirmationModal({
                    open: true,
                    args: {
                        emtity: null
                        toAddress: approval.token as Hex,
                        chainId,
                        calldata: approval.calldata as `0x${string}`,
                        onConfirm: (hash: Hex) => {
                            deps.manager.createApprovalTransaction({
                                ...deps.args,
                                txHash: hash,
                                networkKey: network,
                                queryKey: '',
                                entity: null,
                                subgraphUrl: ''
                            });
                        },
                    }
                });
            }




    

}

const handleDeploymentTransaction = async (deps: HandleAddOrderDependencies) => {
		for (const approval of approvals) {
			let approvalHash: Hex;
			try {
				awaitWalletConfirmation(
					`Please approve ${approval.symbol || approval.token} spend in your wallet...`
				);
				approvalHash = await sendTransaction(config, {
					to: approval.token as `0x${string}`,
					data: approval.calldata as `0x${string}`
				});
			} catch {
				return transactionError(TransactionErrorMessage.USER_REJECTED_APPROVAL);
			}
			try {
				awaitApprovalTx(approvalHash, approval.symbol);
				await waitForTransactionReceipt(config, { hash: approvalHash });
			} catch {
				return transactionError(TransactionErrorMessage.APPROVAL_FAILED);
			}
		}

		let hash: Hex;
		try {
			awaitWalletConfirmation('Please confirm deployment in your wallet...');
			hash = await sendTransaction(config, {
				to: orderbookAddress as `0x${string}`,
				data: deploymentCalldata as `0x${string}`
			});
		} catch {
			return transactionError(TransactionErrorMessage.USER_REJECTED_TRANSACTION);
		}
		try {
			const transactionExplorerLink = await getExplorerLink(hash, chainId, 'tx');
			awaitTx(hash, TransactionStatusMessage.PENDING_DEPLOYMENT, transactionExplorerLink);
			await waitForTransactionReceipt(config, { hash });
			if (subgraphUrl) {
				return awaitNewOrderIndexing(subgraphUrl, hash, network);
			}
			return transactionSuccess(
				hash,
				'Deployment successful. Check the Orders page for your new order.',
				'',
				network
			);
		} catch {
			return transactionError(TransactionErrorMessage.DEPLOYMENT_FAILED);
		}
	};