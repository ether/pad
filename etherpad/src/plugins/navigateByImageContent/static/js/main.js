import("sqlbase.sqlobj");
import("sqlbase.sqlbase");
import("etherpad.log");
import("etherpad.utils.*");
import("plugins.padHierarchy.helpers.hierarchyHelper.*");

function navigateByImageContentInit() {
  this.hooks = [];
}

navigateByImageContent = new navigateByImageContentInit();
