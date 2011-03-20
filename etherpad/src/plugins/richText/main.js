import("etherpad.log");
import("plugins.richText.hooks");
import("plugins.richText.static.js.main");

function richTextInit() {
 this.hooks = [ 'editBarItemsLeftPad', 'aceAttribsToClasses', 'aceCreateDomLine'];
 this.description = 'Rich Text editing in etherpad';
 this.client = new main.richTextInit();
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

