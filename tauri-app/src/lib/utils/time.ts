import dayjs from 'dayjs';
import bigIntSupport from 'dayjs/plugin/bigIntSupport';
import localizedFormat from 'dayjs/plugin/localizedFormat';
import type { UTCTimestamp } from 'lightweight-charts';
dayjs.extend(bigIntSupport);
dayjs.extend(localizedFormat);

export function formatTimestampSecondsAsLocal(timestampSeconds: bigint) {
  return dayjs(timestampSeconds  * BigInt('1000'))
    .format('L LT');
}

export function timestampSecondsToUTCTimestamp(timestampSeconds: bigint) {
  return dayjs(timestampSeconds  * BigInt('1000')).unix() as UTCTimestamp
}


/**
 * Method to put a timeout on a promise, throws the exception if promise is not settled within the time
 * @returns A new promise that gets settled with initial promise settlement or rejected with exception value
 * if the time runs out before the main promise settlement
 */
export async function promiseTimeout<T>(promise: Promise<T>, time: number, exception: string | number | bigint | symbol | boolean): Promise<T> {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  let timeout: any;
  return Promise.race([
      promise,
      new Promise(
          (_res, _rej) => timeout = setTimeout(_rej, time, exception)
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      ) as any
  ]).finally(
      () => clearTimeout(timeout)
  );
};