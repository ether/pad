import("etherpad.log");
import("plugins.debugRaiseException.hooks");
import("plugins.debugRaiseException.static.js.main");

function debugRaiseExceptionInit() {
 this.hooks = ['editBarItemsLeftPad'];
 this.description = 'Raise an exception at the click of a button';
 this.client = new main.debugRaiseExceptionInit();
 this.editBarItemsLeftPad = hooks.editBarItemsLeftPad;
 this.install = install;
 this.uninstall = uninstall;
}

function install() {
 log.info("Installing debugRaiseException");
}

function uninstall() {
 log.info("Uninstalling debugRaiseException");
}

