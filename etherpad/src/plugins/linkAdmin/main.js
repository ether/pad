import("etherpad.log");
import("plugins.linkAdmin.hooks");

function linkAdminInit() {
 this.hooks = ['docbarItemsTagBrowser'];
 this.description = 'Link to the Admin page from the Tag Browser home page.';
 // this.client = new main.linkAdminInit();
 this.docbarItemsTagBrowser = hooks.docbarItemsTagBrowser;
 this.install = install;
 this.uninstall = uninstall;
}

function install() {
 log.info("Installing linkAdmin");
}

function uninstall() {
 log.info("Uninstalling linkAdmin");
}

