import("etherpad.log");
import("plugins.deletePad.hooks");
import("sqlbase.sqlobj");
import("sqlbase.sqlcommon");

function deletePadInit() {
 this.hooks = ['handlePath'];
 this.description = 'Allows admin to delete pads permanently';
 this.configLink = '/ep/admin/delete-pad';
 this.handlePath = hooks.handlePath;

 this.install = install;
 this.uninstall = uninstall;
}

function install() {
 log.info("Installing delete pad");
}

function uninstall() {
 log.info("Uninstalling delete pad");
}

