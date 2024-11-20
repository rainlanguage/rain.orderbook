import dayjs from 'dayjs';
import bigIntSupport from 'dayjs/plugin/bigIntSupport';
import localizedFormat from 'dayjs/plugin/localizedFormat';
import type { UTCTimestamp } from 'lightweight-charts';
dayjs.extend(bigIntSupport);
dayjs.extend(localizedFormat);

export function formatTimestampSecondsAsLocal(timestampSeconds: bigint) {
	return dayjs(timestampSeconds * BigInt('1000')).format('L LT');
}

export function timestampSecondsToUTCTimestamp(timestampSeconds: bigint) {
	return dayjs(timestampSeconds * BigInt('1000')).unix() as UTCTimestamp;
}

/**
 * Method to put a timeout on a promise, throws the exception if promise is not settled within the time
 * @param promise - The original promise
 * @param time - The time in ms
 * @param exception - The exception to reject with if time runs out before original promise settlement
 * @returns A new promise that gets settled with initial promise settlement or rejected with exception value
 * if the time runs out before the main promise settlement
 */
export async function promiseTimeout<T>(
	promise: Promise<T>,
	time: number,
	exception: unknown
): Promise<T> {
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	let timeout: any;
	return Promise.race([
		promise,
		new Promise((_resolve, reject) => (timeout = setTimeout(reject, time, exception))) as Promise<T>
	]).finally(() => clearTimeout(timeout));
}
