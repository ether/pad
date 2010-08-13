import("etherpad.log");
import("plugins.padHierarchy.static.js.main");
import("plugins.padHierarchy.hooks");
function init() {
 this.hooks = ['handlePath'];
 this.client = new main.init(); 
 this.description = 'Allows groups of documents to be found and created based on url structure.';
 //this.renderPageBodyPre = main.renderPageBodyPre;
 this.handlePath = hooks.handlePath;
 this.install = install;
 this.uninstall = uninstall;
}

function install() {
 log.info("Installing padHierarchy");
}

function uninstall() {
 log.info("Uninstalling padHierarchy");
}
