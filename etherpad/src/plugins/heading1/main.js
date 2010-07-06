import("etherpad.log");
import("plugins.heading1.hooks");
import("plugins.heading1.static.js.main");

function heading1Init() {
 this.hooks = ['editBarItemsLeftPad', 'aceAttribsToClasses', 'aceCreateDomLine'];
 this.description = 'heading1';
 this.client = new main.init();
 this.editBarItemsLeftPad = hooks.editBarItemsLeftPad;
 this.aceAttribsToClasses = main.aceAttribsToClasses;
 this.aceCreateDomLine = main.aceCreateDomLine;
 this.install = install;
 this.uninstall = uninstall;
}

function install() {
 log.info("Installing heading1");
}

function uninstall() {
 log.info("Uninstalling heading1");
}

