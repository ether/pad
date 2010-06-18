import("etherpad.log");
import("plugins.realTimeRecentChanges.static.js.main");

function init() {
 this.hooks = ['handlePath'];
 this.client = new main.init();
 this.description = 'Real-Time Recent Changes modifies the twitterStyleTags Tag Browser to update in real time.';
 this.aceGetFilterStack = main.aceGetFilterStack;
 this.aceCreateDomLine = main.aceCreateDomLine;

 this.handlePath = hooks.handlePath;

 this.install = install;
 this.uninstall = uninstall;
}

function install() {
 log.info("Installing Real-Time Recent Changes plugin.");
}

function uninstall() {
 log.info("Uninstalling Real-Time Recent Changes plugin.");
}

