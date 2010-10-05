import("etherpad.log");
import("plugins.licenseTag.hooks");
import("plugins.licenseTag.static.js.main");

function licenseTagInit() {
 this.hooks = ['editBarItemsLeftPad'];
 this.description = 'License tag inserter (for twitterStyleTags).';
 this.client = new main.licenseTagInit();
 this.editBarItemsLeftPad = hooks.editBarItemsLeftPad;
 this.install = install;
 this.uninstall = uninstall;
}

function install() {
 log.info("Installing licenseTag");
}

function uninstall() {
 log.info("Uninstalling licenseTag");
}

