// ==UserScript==
// @name          Gravpad
// @namespace     http://metameso.org/
// @description   An iframe appears on every page, containing an etherpad!
// @exclude       *metameso.org:9000*
// @exclude       *doc.etherpad.org*
// @exclude       about:blank
// @version       0.1
// ==/UserScript==

// It seems necessary to exclude most versions of
// Etherpad, apparently because an extra iframe triggers
// greasemonkey.

// We do something here to modify the CSS of the main page.
// Note that this is a bit sloppy, in that ideally we would reroute
// the main page into an iframe of its own, to keep things cleaner.

GM_addStyle("body { float: left; width: 50%; }");
// Now create a new iframe
var newTop = document.createElement("iframe");
newTop.setAttribute("id", "addedPad");
newTop.setAttribute("name", "addedPad");
newTop.setAttribute("frameborder", "yes");
newTop.setAttribute("height", "887px");
newTop.setAttribute("width", "40%");
newTop.setAttribute("style", "float: right; position: fixed; z-index: 9999; right: 1px;");
newTop.setAttribute("src", "http://metameso.org:9000/" +
                     window.location.href.replace(/\//gi,"-"));
document.documentElement.appendChild(newTop);


