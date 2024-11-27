'use strict';

var core = require('@tauri-apps/api/core');

exports.RelayError = void 0;
(function (RelayError) {
    RelayError["InvalidMethod"] = "InvalidMethod";
    RelayError["InvalidUrl"] = "InvalidUrl";
    RelayError["InvalidHeaders"] = "InvalidHeaders";
    RelayError["RequestCancelled"] = "RequestCancelled";
    RelayError["RequestRunError"] = "RequestRunError";
})(exports.RelayError || (exports.RelayError = {}));
async function run(options) {
    return await core.invoke('plugin:hoppscotch-relay|run', { options });
}
async function cancel(options) {
    return await core.invoke('plugin:hoppscotch-relay|cancel', { options });
}

exports.cancel = cancel;
exports.run = run;
