export enum TransactionName {
	REMOVAL = 'Order Removal',
	WITHDRAWAL = 'Vault Withdrawal',
	APPROVAL = 'Token Approval',
	DEPOSIT = 'Vault Deposit'
}

/**
 * Creates a formatted token approval message
 * @param tokenSymbol The token symbol (e.g., "DAI")
 * @returns A formatted approval message string
 */
export function getApprovalMessage(tokenSymbol: string): string {
	return `Approving ${tokenSymbol} spend`;
}

/**
 * Creates a formatted vault deposit message
 * @param tokenSymbol The token symbol (e.g., "DAI")
 * @returns A formatted deposit message string
 */
export function getDepositMessage(tokenSymbol: string): string {
	return `Depositing ${tokenSymbol} to vault`;
}
