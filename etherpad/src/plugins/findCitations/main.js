import("etherpad.log");
import("plugins.findCitations.hooks");

function findCitationsInit() {
 this.hooks = ['handlePath', 'docbarItemsTagBrowser'];
 this.description = 'Use data created by urlIndexer, but now search by URL instead of by tag.';

 /* These hooks just create a simple browsing interface
    and a link to this interface in the tag browser. */

 this.handlePath = hooks.handlePath;

 /* This shows up as an option when we're in the tag browser-based plugins.  */
 this.docbarItemsTagBrowser = hooks.docbarItemsTagBrowser;

 this.install = install;
 this.uninstall = uninstall;
}

function install() {
 log.info("Installing findCitations");
}

function uninstall() {
 log.info("Uninstalling findCitations");
}

