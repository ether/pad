import("etherpad.log");
import("plugins.debugFirebugLite.hooks");
import("plugins.debugFirebugLite.static.js.main");

function debugFirebugLiteInit() {
 this.hooks = ['handlePath'];
 this.description = 'Firebug Lite';
 this.client = new main.debugFirebugLiteInit();
 this.handlePath = hooks.handlePath;
 this.install = install;
 this.uninstall = uninstall;
}

function install() {
 log.info("Installing debugFirebugLite");
}

function uninstall() {
 log.info("Uninstalling debugFirebugLite");
}

