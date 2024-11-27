import { invoke } from '@tauri-apps/api/core';

var RelayError;
(function (RelayError) {
    RelayError["InvalidMethod"] = "InvalidMethod";
    RelayError["InvalidUrl"] = "InvalidUrl";
    RelayError["InvalidHeaders"] = "InvalidHeaders";
    RelayError["RequestCancelled"] = "RequestCancelled";
    RelayError["RequestRunError"] = "RequestRunError";
})(RelayError || (RelayError = {}));
async function run(options) {
    return await invoke('plugin:hoppscotch-relay|run', { options });
}
async function cancel(options) {
    return await invoke('plugin:hoppscotch-relay|cancel', { options });
}

export { RelayError, cancel, run };
