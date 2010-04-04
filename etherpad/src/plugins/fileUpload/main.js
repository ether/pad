import("etherpad.log");
import("plugins.fileUpload.hooks");

function init() {
 this.hooks = ['handlePath'];
 this.description = 'File upload manager';
 this.handlePath = hooks.handlePath;
 this.install = install;
 this.uninstall = uninstall;
}

function install() {
 log.info("Installing fileUpload");
}

function uninstall() {
 log.info("Uninstalling fileUpload");
}

