import("etherpad.log");
import("plugins.showImages.static.js.main");

function init() {
 this.hooks = ['aceGetFilterStack', 'aceCreateDomLine'];
 this.client = new main.init(); 
 this.description = 'Render images inline';
 this.aceGetFilterStack = main.aceGetFilterStack;
 this.aceCreateDomLine = main.aceCreateDomLine;
 this.install = install;
 this.uninstall = uninstall;
}

function install() {
 log.info("Installing inlineImages");
}

function uninstall() {
 log.info("Uninstalling inlineImages");
}
