import("etherpad.log");
import("plugins.sketchSpace.hooks");
import("plugins.sketchSpace.static.js.main");

function sketchSpaceInit() {
 this.hooks = ['editBarItemsLeftPad', 'aceAttribsToClasses', 'aceCreateDomLine', 'modals'];
 this.description = 'sketchSpace';
 this.client = new main.sketchSpaceInit();
 this.editBarItemsLeftPad = hooks.editBarItemsLeftPad;
 this.aceAttribsToClasses = main.aceAttribsToClasses;
 this.aceCreateDomLine = main.aceCreateDomLine;
 this.modals = hooks.modals;
 this.install = install;
 this.uninstall = uninstall;
}

function install() {
 log.info("Installing sketchSpace");
}

function uninstall() {
 log.info("Uninstalling sketchSpace");
}

