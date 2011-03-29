import("etherpad.log");
import("faststatic");
import("etherpad.utils.*");
import("etherpad.globals.*");
import("dispatch.{Dispatcher,PrefixMatcher,forward}");
import("plugins.imageConvert.controllers.imageConvert");

function handlePath() {
  return [[PrefixMatcher('/ep/imageConvert/'), forward(imageConvert)]];
}
