import("etherpad.log");
import("plugins.realTimeRecentChanges.hooks");
import("plugins.realTimeRecentChanges.static.js.main");

function realTimeRecentChangesInit() {
 this.hooks = ['handlePath', 'docbarItemsTagBrowser'];
 this.client = new main.realTimeRecentChangesInit();
 this.description = 'Real-Time Recent Changes modifies the twitterStyleTags Tag Browser to update in real time.';

 this.handlePath = hooks.handlePath;
 this.docbarItemsTagBrowser = hooks.docbarItemsTagBrowser;

 this.install = install;
 this.uninstall = uninstall;
}

function install() {
 log.info("Installing Real-Time Recent Changes plugin.");
}

function uninstall() {
 log.info("Uninstalling Real-Time Recent Changes plugin.");
}

