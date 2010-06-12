import("etherpad.log");
import("plugins.wikiStyleLinks.static.js.main");

/* initialization function for the plugin */

/* creates standard methods/members -- hooks and
   description are required; hooks are a list of hook
   names -- available hook names are documented on
   http://doc.etherpad.org/... register these, and then
   provide methods w/ the same names;
     e.g. handlePath is done via
     hooks.handlePath, defined in hooks in this directory! 
   install and uninstall are methods you don't have to
   register in this.hooks 

   this.client = new main.init() is what runs on the
   client. What goes on is that ./static is served to the
   client, and everything there is run on the client side,
   but main.init is run on the server side AS WELL to
   establish the connection between client and server. */

function init() {
 this.hooks = ['aceGetFilterStack', 'aceCreateDomLine'];
 this.client = new main.init();
 this.description = 'Wiki-style links lets the user link from Pad A to Pad B by writing [[name of Pad B]] in Pad A.';
 this.aceGetFilterStack = main.aceGetFilterStack;
 this.aceCreateDomLine = main.aceCreateDomLine;

 this.install = install;
 this.uninstall = uninstall;
}

/* This does things like create database tables (or create
   directories or get an API or whatever); general setup
   stuff. */

function install() {
 log.info("Installing Wiki-style links");
}

/* You probably don't want uninstalling to delete data.
   But you can do something here if you want. */

function uninstall() {
 log.info("Uninstalling Wiki-style links");
}

