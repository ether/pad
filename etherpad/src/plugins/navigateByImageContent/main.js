import("etherpad.log");
import("plugins.navigateByImageContent.static.js.main");
import("plugins.navigateByImageContent.hooks");
function init() {
 this.hooks = ['renderNavigation','padModelWriteToDB'];
 this.client = new main.init(); 
 this.description = 'Adds a header section where all documents can be traversed horizontally by the images they contain.';
 this.renderNavigation = main.renderNavigation;
 this.padModelWriteToDB = hooks.padModelWriteToDB;
 this.install = install;
 this.uninstall = uninstall;
}

function install() {
 log.info("Installing navigateByImageContent");
}

function uninstall() {
 log.info("Uninstalling navigateByImageContent");
}
