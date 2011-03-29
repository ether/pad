import("etherpad.log");
import("plugins.imageConvert.hooks");

function imageConvertInit() {
 this.hooks = ['handlePath'];
 this.description = 'Plugin for fileUpload that supports on-the-fly image resizeing and conversion';
 this.handlePath = hooks.handlePath;
 this.install = install;
 this.uninstall = uninstall;
}

function install() {
 log.info("Installing imageConvert");
}

function uninstall() {
 log.info("Uninstalling imageConvert");
}

