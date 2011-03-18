import("etherpad.log");
import("plugins.openingDesign.hooks");
import("plugins.openingDesign.static.js.main");

function openingDesignInit() {
 this.hooks = ['editBarItemsLeftPad', 'aceAttribsToClasses', 'aceCreateDomLine', 'modals'];
 this.description = 'openingDesign';
 this.client = new main.openingDesignInit();
 this.editBarItemsLeftPad = hooks.editBarItemsLeftPad;
 this.aceAttribsToClasses = main.aceAttribsToClasses;
 this.aceCreateDomLine = main.aceCreateDomLine;
 this.modals = hooks.modals;
 this.install = install;
 this.uninstall = uninstall;
}

function install() {
 log.info("Installing openingDesign");
}

function uninstall() {
 log.info("Uninstalling openingDesign");
}

