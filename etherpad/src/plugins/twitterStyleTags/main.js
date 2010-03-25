import("etherpad.log");
import("plugins.twitterStyleTags.hooks");
import("plugins.twitterStyleTags.static.js.main");

function init() {
 this.hooks = ['handlePath'];
 this.client = new main.init();
 this.description = 'Twitter-style tags';
 this.handlePath = hooks.handlePath;
 this.aceGetFilterStack = hooks.aceGetFilterStack;

 this.install = install;
 this.uninstall = uninstall;
}

function install() {
 log.info("Installing Twitter-style tags");
}

function uninstall() {
 log.info("Uninstalling Twitter-style tags");
}

