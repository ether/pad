import("etherpad.log");
import("faststatic");
import("etherpad.utils.*");
import("etherpad.globals.*");
import("etherpad.helpers");

function handlePath(arg) {
  helpers.includeCss('plugins/debugJsBacktrace/javascript-stacktrace/qunit.css');
  helpers.includeJs('plugins/debugJsBacktrace/javascript-stacktrace/qunit.js');
  helpers.includeJs('plugins/debugJsBacktrace/javascript-stacktrace/stacktrace.js');
  helpers.includeJs('plugins/debugJsBacktrace/load.js');
}
