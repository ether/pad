import("etherpad.log");
import("dispatch.{Dispatcher,PrefixMatcher,forward}");
import("plugins.search.controllers.browser");
import("sqlbase.sqlobj");

function handlePath() {
  return [[PrefixMatcher('/ep/search'), forward(browser)]];
}

function docbarItemsAll() {
 return ["<td class='docbarbutton highlight'><a href='/ep/search'><img src='/static/img/plugins/search/icon_home.gif'>Home</a></td>"];
}

function editBarItemsLeftPad(arg) {
  return arg.template.include('searchEditbarButtons.ejs', undefined, ['search']);
}
