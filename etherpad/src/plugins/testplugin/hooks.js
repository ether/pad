import("etherpad.log");
import("dispatch.{Dispatcher,PrefixMatcher,forward}");
import("plugins.testplugin.controllers.testplugin");

function serverStartup() {
 log.info("Server startup for testplugin");
}

function serverShutdown() {
 log.info("Server shutdown for testplugin");
}

function handlePath() {
 return [[PrefixMatcher('/ep/testplugin/'), forward(testplugin)]];
}
