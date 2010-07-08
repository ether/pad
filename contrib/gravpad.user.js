// ==UserScript==
// @name          Gravpad
// @namespace     http://metameso.org/
// @description   An iframe appears on every page, containing an etherpad!
// @exclude       *metameso.org:9000*
// @exclude       about:blank
// @version       0.1
// ==/UserScript==

if (window != top) return; // <-- suggested as a way to prevent running on sub-iframes.

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
newTop.setAttribute("height", "787px");
newTop.setAttribute("width", "40%");
newTop.setAttribute("style", "float: right; position: fixed; z-index: 999; right: 1px;");
newTop.setAttribute("src", "http://metameso.org:9000/" +
                     window.location.href.replace(/\//gi,"-"));
document.documentElement.appendChild(newTop);


var rtrc = document.createElement("iframe");
rtrc.setAttribute("id", "addedChanges");
rtrc.setAttribute("name", "addedChanges");
rtrc.setAttribute("frameborder", "yes");
rtrc.setAttribute("height", "207px");
rtrc.setAttribute("width", "40%");
rtrc.setAttribute("style", "float: right; position: fixed; bottom: 0; fixed; z-index: 9999; right: 1px;");
rtrc.setAttribute("src", "http://metameso.org:9000/ep/rtrc/");
document.documentElement.appendChild(rtrc);
