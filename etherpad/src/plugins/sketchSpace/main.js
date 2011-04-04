import("etherpad.log");
import("plugins.sketchSpace.hooks");
import("plugins.sketchSpace.static.js.main");

function sketchSpaceInit() {
 this.hooks = ['editBarItemsLeftPad', 'aceAttribsToClasses', 'aceCreateDomLine', 'modals', 'editBarItemsLeftPadView'];
 this.description = 'sketchSpace';
 this.client = new main.sketchSpaceInit();
 this.editBarItemsLeftPad = hooks.editBarItemsLeftPad;
 this.editBarItemsLeftPadView = hooks.editBarItemsLeftPadView;
 this.aceAttribsToClasses = this.client.aceAttribsToClasses;
 this.aceCreateDomLine = this.client.aceCreateDomLine;
 this.modals = hooks.modals;
 this.install = install;
 this.uninstall = uninstall;

 this.editorArea = {images:[], currentImage: undefined};
}

function install() {
 log.info("Installing sketchSpace");
}

function uninstall() {
 log.info("Uninstalling sketchSpace");
}

