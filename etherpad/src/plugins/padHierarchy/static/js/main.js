import("sqlbase.sqlobj");
import("sqlbase.sqlbase");
import("etherpad.log");

function padHierarchyInit() {
  this.hooks = [];//['renderPageBodyPre'];
}
padHierarchy = new padHierarchyInit();
