import("etherpad.log");
import("plugins.openingDesign.hooks");

function openingDesignInit() {
 this.hooks = ['modals', 'docbarItemsPad'];
 this.description = 'openingDesign';
 this.docbarItemsPad = hooks.docbarItemsPad;
 this.modals = hooks.modals;
 this.install = install;
 this.uninstall = uninstall;
}

function install() {
 log.info("Installing openingDesign");
}

function uninstall() {
 log.info("Uninstalling openingDesign");
}

