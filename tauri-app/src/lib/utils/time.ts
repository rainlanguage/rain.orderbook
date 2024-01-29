import dayjs from 'dayjs';
import utc from 'dayjs/plugin/utc';
import bigIntSupport from 'dayjs/plugin/bigIntSupport';
dayjs.extend(utc);
dayjs.extend(bigIntSupport);

export function formatTimestampSecondsAsLocal(timestampSeconds: bigint) {
  return dayjs(timestampSeconds  * BigInt('1000'))
    .utc(true)
    .local()
    .format('DD/MM/YYYY h:mm A');
}