import("etherpad.log");
import("dispatch.{Dispatcher,PrefixMatcher,forward}");
import("plugins.deletePad.controllers.ui");
import("sqlbase.sqlobj");

function handlePath() {
  return [[PrefixMatcher('/ep/admin/delete-pad'), forward(ui)]];
}
