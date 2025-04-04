import type { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';
import {
	getDeploymentTransactionArgs,
	type HandleAddOrderResult
} from '../components/deployment/getDeploymentTransactionArgs';
import type { Account } from '$lib/types/account';
import type { DeploymentArgs } from '../types/transaction';
import { get } from 'svelte/store';

export interface DeploymentHandlers {
	handleDisclaimerModal?: (props: { open: boolean; onAccept: () => void }) => void;
	handleDeployModal: (props: { open: boolean; args: DeploymentArgs }) => void;
}

export async function handleDeployment(
	gui: DotrainOrderGui,
	account: string,
	handlers: DeploymentHandlers,
	subgraphUrl?: string
): Promise<void> {
	const result: HandleAddOrderResult = await getDeploymentTransactionArgs(gui, account);

	if (handlers.handleDisclaimerModal) {
		handlers.handleDisclaimerModal({
			open: true,
			onAccept: () => {
				handlers.handleDeployModal({
					open: true,
					args: {
						...result,
						subgraphUrl
					}
				});
			}
		});
	} else {
		handlers.handleDeployModal({
			open: true,
			args: {
				...result,
				subgraphUrl
			}
		});
	}
}
