import dayjs from "dayjs";
import bigIntSupport from "dayjs/plugin/bigIntSupport.js";
import localizedFormat from "dayjs/plugin/localizedFormat.js";
import utc from "dayjs/plugin/utc.js";
dayjs.extend(bigIntSupport);
dayjs.extend(localizedFormat);
dayjs.extend(utc);
function formatTimestampSecondsAsLocal(timestampSeconds) {
  return dayjs(timestampSeconds * BigInt("1000")).utc().format("L LT");
}
function timestampSecondsToUTCTimestamp(timestampSeconds) {
  return dayjs(timestampSeconds * BigInt("1000")).unix();
}
async function promiseTimeout(promise, time, exception) {
  let timeout;
  return Promise.race([
    promise,
    new Promise((_resolve, reject) => timeout = setTimeout(reject, time, exception))
  ]).finally(() => clearTimeout(timeout));
}
export {
  formatTimestampSecondsAsLocal as f,
  promiseTimeout as p,
  timestampSecondsToUTCTimestamp as t
};
//# sourceMappingURL=time.js.map
