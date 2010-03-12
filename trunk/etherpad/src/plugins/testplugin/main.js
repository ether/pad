import("etherpad.log");
import("etherpad.admin.plugins");

hooks = ['testhook', 'nahook', 'serverStartup', 'serverShutdown'];
description = 'Test Plugin';

function install() {
 log.info("Installing testplugin");
}

function uninstall() {
 log.info("Uninstalling testplugin");
}

function testhook () {
}

function nahook() {
}

function serverStartup() {
 log.info("Server startup for testplugin");
}

function serverShutdown() {
 log.info("Server shutdown for testplugin");
}
