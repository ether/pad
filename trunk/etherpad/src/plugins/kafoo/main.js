import("etherpad.log");

function init() {
 this.hooks = [];
 this.description = 'KaBar plugin';
 this.install = install;
 this.uninstall = uninstall;
}

function install() {
 log.info("Installing testplugin");
}

function uninstall() {
 log.info("Uninstalling testplugin");
}
