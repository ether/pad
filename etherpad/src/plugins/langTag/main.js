import("etherpad.log");
import("plugins.langTag.hooks");
import("plugins.langTag.static.js.main");

function langTagInit() {
 this.hooks = ['editBarItemsLeftPad'];
 this.description = 'Language tag inserter (for twitterStyleTags).';
 this.client = new main.langTagInit();
 this.editBarItemsLeftPad = hooks.editBarItemsLeftPad;
 this.install = install;
 this.uninstall = uninstall;
}

function install() {
 log.info("Installing langTag");
}

function uninstall() {
 log.info("Uninstalling langTag");
}

