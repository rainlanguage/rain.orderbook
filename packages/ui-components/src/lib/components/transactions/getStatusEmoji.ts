import { TransactionStatusMessage } from '$lib/types/transaction';
import { match } from 'ts-pattern';

export function getStatusEmoji(status: TransactionStatusMessage): string {
	return match(status)
		.with(TransactionStatusMessage.PENDING_RECEIPT, () => '🔄')
		.with(TransactionStatusMessage.PENDING_SUBGRAPH, () => '📊')
		.with(TransactionStatusMessage.SUCCESS, () => '✅')
		.with(TransactionStatusMessage.ERROR, () => '❌')
		.otherwise(() => '❓');
}

if (import.meta.vitest) {
	describe('getStatusEmoji', () => {
		it('should return the correct emoji for each status', () => {
			expect(getStatusEmoji(TransactionStatusMessage.PENDING_RECEIPT)).toBe('🔄');
			expect(getStatusEmoji(TransactionStatusMessage.PENDING_SUBGRAPH)).toBe('📊');
			expect(getStatusEmoji(TransactionStatusMessage.SUCCESS)).toBe('✅');
			expect(getStatusEmoji(TransactionStatusMessage.ERROR)).toBe('❌');
		});
	});
}
