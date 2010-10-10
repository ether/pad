import("etherpad.log");
import("plugins.copyPad.hooks");

function copyPadInit() {
 this.hooks = ['handlePath', 'editBarItemsRightPad', 'editBarItemsRightPadView'];
 this.description = 'Lets users copy pads';
 this.handlePath = hooks.handlePath;
 this.editBarItemsRightPad = hooks.editBarItemsRightPad;
 this.editBarItemsRightPadView = hooks.editBarItemsRightPad;
 this.install = install;
 this.uninstall = uninstall;
}

function install() {
 log.info("Installing copyPad");
}

function uninstall() {
 log.info("Uninstalling copyPad");
}

