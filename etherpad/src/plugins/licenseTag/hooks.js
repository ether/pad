import("etherpad.log");
import("faststatic");
import("etherpad.utils.*");
import("etherpad.globals.*");

function tagSelectors(arg) {
  return [arg.template.include('licenseTagTagSelectors.ejs', undefined, ['licenseTag'])];
}
