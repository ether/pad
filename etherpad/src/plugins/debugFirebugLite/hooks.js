import("etherpad.log");
import("faststatic");
import("etherpad.utils.*");
import("etherpad.globals.*");
import("etherpad.helpers");

function handlePath(arg) {
  helpers.addToHead("<script src='/static/js/plugins/debugFirebugLite/firebug-lite.js'></script>");
}
