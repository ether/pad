/**
 * THIS PLUGIN IS NOT READY TO BE MERGED JUST YET...
 * 
 * This plugin allows documents to be arranged hierarchically
 * and presented independently of the editing environment in
 * a variety of formats.
 * 
 * A goal is to limit access to the edit mode (denoted by urls
 * ending with /+edit ) to a set of contributors based
 * on openid-based authentication.
 *  
 */

import("etherpad.log");
import("plugins.padHierarchy.static.js.main");
import("plugins.padHierarchy.hooks");
function init() {
 this.hooks = ['handlePath'];
 this.client = new main.init(); 
 this.description = 'Allows groups of documents to be found and created based on url structure.';
 //this.renderPageBodyPre = main.renderPageBodyPre;
 this.handlePath = hooks.handlePath;
 this.install = install;
 this.uninstall = uninstall;
}

function install() {
 log.info("Installing padHierarchy");
}

function uninstall() {
 log.info("Uninstalling padHierarchy");
}
