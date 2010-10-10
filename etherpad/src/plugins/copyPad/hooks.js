import("etherpad.log");
import("faststatic");
import("etherpad.utils.*");
import("etherpad.globals.*");
import("dispatch.{Dispatcher,PrefixMatcher,forward}");
import("plugins.copyPad.controllers.copyPad");

function handlePath() {
  return [[PrefixMatcher('/ep/copyPad'), forward(copyPad)]];
}

function editBarItemsRightPad(arg) {
  return arg.template.include('copyPadEditbarButtons.ejs', undefined, ['copyPad']);
}
