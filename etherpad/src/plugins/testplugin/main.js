import("etherpad.log");
import("plugins.testplugin.hooks");
import("plugins.testplugin.static.js.main");

function testpluginInit() {
 this.hooks = ['serverStartup', 'serverShutdown', 'handlePath'];
 this.client = new main.testpluginInit();
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

