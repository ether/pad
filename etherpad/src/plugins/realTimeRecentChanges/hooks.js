import("etherpad.log");
import("dispatch.{Dispatcher,PrefixMatcher,forward}");
import("plugins.realTimeRecentChanges.controllers.rtrcBrowser");

function handlePath() {
  return [[PrefixMatcher('/ep/rtrc'), forward(rtrcBrowser)]];
}

function docbarItemsSearch() {
 return ["<td class='docbarbutton'><a href='/ep/rtrc/'>RTRC</a></td>"];
}

