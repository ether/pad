import("etherpad.log");
import("faststatic");
import("etherpad.utils.*");
import("etherpad.globals.*");
import("dispatch.{Dispatcher,PrefixMatcher,forward}");

function editBarItemsLeftPad(arg) {
  return arg.template.include('sketchSpaceEditbarButtons.ejs', undefined, ['sketchSpace']);
}

function modals(arg) {
  return arg.template.include('sketchSpaceModals.ejs', undefined, ['sketchSpace']);
}
