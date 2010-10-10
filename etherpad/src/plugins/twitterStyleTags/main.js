import("etherpad.log");
import("plugins.twitterStyleTags.hooks");
import("plugins.twitterStyleTags.static.js.main");
import("sqlbase.sqlobj");
import("sqlbase.sqlcommon");

function twitterStyleTagsInit() {
 this.hooks = ['handlePath', 'aceGetFilterStack', 'aceCreateDomLine', 'padModelWriteToDB', 'docbarItemsAll', 'docbarItemsTagBrowser', 'editBarItemsLeftPad'];
 this.client = new main.twitterStyleTagsInit();
 this.description = 'Twitter-style tags allows the user to tag pads by writing #tagname anywhere in the pad text. Tags are automatically linked to searches for that tag in other pads. This plugin also provides an alternative home-page for Etherpad with a display of the last changed public pads as well as that information available as an RSS stream.';
 this.handlePath = hooks.handlePath;
 this.aceGetFilterStack = main.twitterStyleTagsInit.prototype.aceGetFilterStack;
 this.aceCreateDomLine = main.twitterStyleTagsInit.prototype.aceCreateDomLine;
 this.padModelWriteToDB = hooks.padModelWriteToDB;
 this.docbarItemsAll = hooks.docbarItemsAll;
 this.docbarItemsTagBrowser = hooks.docbarItemsTagBrowser;
 this.editBarItemsLeftPad = hooks.editBarItemsLeftPad;

 this.install = install;
 this.uninstall = uninstall;
}

function install() {
 log.info("Installing Twitter-style tags");

 sqlobj.createTable('TAG', {
   ID: 'int not null '+sqlcommon.autoIncrementClause()+' primary key',
   NAME: 'varchar(128) character set utf8 collate utf8_bin not null',
  });

 sqlobj.createTable('PAD_TAG', {
   PAD_ID: 'varchar(128) character set utf8 collate utf8_bin not null references PAD_META(ID)',
   TAG_ID: 'int default NULL references TAG(ID)',
  });

 sqlobj.createTable('PAD_TAG_CACHE', {
   PAD_ID: 'varchar(128) character set utf8 collate utf8_bin unique not null references PAD_META(ID)',
   TAGS: 'varchar(1024) collate utf8_bin not null',
  });

}

function uninstall() {
 log.info("Uninstalling Twitter-style tags");
}

