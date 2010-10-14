import("etherpad.log");
import("plugins.debugJsBacktrace.hooks");
import("plugins.debugJsBacktrace.static.js.main");

function debugJsBacktraceInit() {
 this.hooks = ['handlePath'];
 this.description = 'Traceback debugging';
 this.client = new main.debugJsBacktraceInit();
 this.handlePath = hooks.handlePath;
 this.install = install;
 this.uninstall = uninstall;
}

function install() {
 log.info("Installing debugJsBacktrace");
}

function uninstall() {
 log.info("Uninstalling debugJsBacktrace");
}

