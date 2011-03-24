import("etherpad.log");
import("plugins.richText.hooks");
import("plugins.richText.static.js.main");

function richTextInit() {
    this.hooks = [ 'editBarItemsLeftPad', 'aceAttribsToClasses', 'aceCreateDomLine', 'aceCreateStructDomLine'];
    this.description = 'Rich Text editing in etherpad';
    this.client = new main.richTextInit();
    this.editBarItemsLeftPad = hooks.editBarItemsLeftPad;
    this.aceAttribsToClasses = main.aceAttribsToClasses;
    this.aceCreateDomLine = main.aceCreateDomLine;
    this.aceCreateStructDomLine = main.aceCreateStructDomLine; 
    this.install = install;
    this.uninstall = uninstall;
}

function install() {
    log.info("Installing richText");
}

function uninstall() {
    log.info("Uninstalling richText");
}

