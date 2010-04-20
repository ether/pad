import("etherpad.log");
import("plugins.urlIndexer.hooks");
import("sqlbase.sqlobj");
import("sqlbase.sqlcommon");

function init() {
 this.hooks = ['padModelWriteToDB', 'handlePath', 'docbarItemsTagBrowser'];
 this.description = 'Indexes URLs linked to in pads so that they can be displayed outside pads, searched for etc.';
 this.padModelWriteToDB = hooks.padModelWriteToDB;
 this.handlePath = hooks.handlePath;
 this.docbarItemsTagBrowser = hooks.docbarItemsTagBrowser;

 this.install = install;
 this.uninstall = uninstall;
}

function install() {
 log.info("Installing urlIndexer");

 sqlobj.createTable('PAD_URL', {
   PAD_ID: 'varchar(128) character set utf8 collate utf8_bin not null references PAD_META(ID)',
   URL: 'varchar(1024) character set utf8 collate utf8_bin not null',
  });

 sqlobj.createTable('PAD_URL_CACHE', {
   PAD_ID: 'varchar(128) character set utf8 collate utf8_bin unique not null references PAD_META(ID)',
   URLS: 'text collate utf8_bin not null',
  });
}

function uninstall() {
 log.info("Uninstalling urlIndexer");
}

