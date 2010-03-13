import("etherpad.log");
import("plugins.testplugin.hooks");

function init() {
 this.hooks = ['serverStartup', 'serverShutdown', 'handlePath'];
 this.description = 'Test Plugin';
 this.serverStartup = hooks.serverStartup;
 this.serverShutdown = hooks.serverShutdown;
 this.handlePath = hooks.handlePath;
 this.install = install;
 this.uninstall = uninstall;
}

function install() {
 log.info("Installing testplugin");
}

function uninstall() {
 log.info("Uninstalling testplugin");
}

