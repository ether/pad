import("etherpad.log");
import("plugins.twitterStyleTags.hooks");
import("plugins.twitterStyleTags.static.js.main");

function init() {
 this.hooks = ['handlePath', 'aceGetFilterStack', 'aceCreateDomLine', 'padModelWriteToDB'];
 this.client = new main.init();
 this.description = 'Twitter-style tags';
 this.handlePath = hooks.handlePath;
 this.aceGetFilterStack = main.aceGetFilterStack;
 this.aceCreateDomLine = main.aceCreateDomLine;
 this.padModelWriteToDB = hooks.padModelWriteToDB;

 this.install = install;
 this.uninstall = uninstall;
}

function install() {
 log.info("Installing Twitter-style tags");
}

function uninstall() {
 log.info("Uninstalling Twitter-style tags");
}

