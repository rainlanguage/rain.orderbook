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
  return dayjs(timestampSeconds  * BigInt('1000')).second() as UTCTimestamp
}