import("etherpad.log");
import("plugins.fileUpload.hooks");

function init() {
 this.hooks = ['handlePath'];
 this.description = 'File upload manager adds a button to pads to upload a file. A URL to the uploaded file is then inserted into the pad.';
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

