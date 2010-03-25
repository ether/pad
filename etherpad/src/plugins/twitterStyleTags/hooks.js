import("etherpad.log");
import("dispatch.{Dispatcher,PrefixMatcher,forward}");
import("plugins.twitterStyleTags.controllers.tagBrowser");

function aceGetFilterStack() {
 log.info("aceGetFilterStack");
 return [];
}

function handlePath() {
 return [[PrefixMatcher('/ep/tags/'), forward(tagBrowser)]];
}
