import dayjs from 'dayjs';
import bigIntSupport from 'dayjs/plugin/bigIntSupport';
import localizedFormat from 'dayjs/plugin/localizedFormat';
dayjs.extend(bigIntSupport);
dayjs.extend(localizedFormat);

export function formatTimestampSecondsAsLocal(timestampSeconds: bigint) {
  return dayjs(timestampSeconds  * BigInt('1000'))
    .format('L LT');
}

export function formatTimestampSecondsAsAsISO(timestampSeconds: bigint) {
  return dayjs(timestampSeconds  * BigInt('1000'))
    .format('YYYY-MM-DD');
}