import("etherpad.log");
import("plugins.linkReadOnly.hooks");

function linkReadOnlyInit() {
 this.hooks = ['docbarItemsPad'];
 this.description = 'Link to Read Only View';
 // this.client = new main.linkReadOnlyInit();
 this.docbarItemsPad = hooks.docbarItemsPad;
 this.install = install;
 this.uninstall = uninstall;
}

function install() {
 log.info("Installing linkReadOnly");
}

function uninstall() {
 log.info("Uninstalling linkReadOnly");
}

