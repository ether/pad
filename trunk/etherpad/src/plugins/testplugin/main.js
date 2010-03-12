import("etherpad.log");
import("etherpad.admin.plugins");
import("dispatch.{Dispatcher,PrefixMatcher,forward}");
import("plugins.testplugin.controllers.testplugin");

hooks = ['serverStartup', 'serverShutdown', 'handlePath'];
description = 'Test Plugin';

function install() {
 log.info("Installing testplugin");
}

function uninstall() {
 log.info("Uninstalling testplugin");
}

function serverStartup() {
 log.info("Server startup for testplugin");
}

function serverShutdown() {
 log.info("Server shutdown for testplugin");
}

function handlePath() {
 return [[PrefixMatcher('/ep/testplugin/'), forward(testplugin)]];
}