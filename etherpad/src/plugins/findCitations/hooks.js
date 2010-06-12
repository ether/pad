import("etherpad.log");
import("dispatch.{Dispatcher,PrefixMatcher,forward}");
import("sqlbase.sqlobj");
import("plugins.findCitations.controllers.citationBrowser");

function handlePath() {
  return [[PrefixMatcher('/ep/citations'), forward(citationBrowser)]];
}

function docbarItemsTagBrowser() {
 return ["<td class='docbarbutton'><a href='/ep/citations/'>Citations</a></td>"];
}

