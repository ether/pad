// ==UserScript==
// @name          Step1 
// @namespace     http://metameso.org/
// @description   Now can we send the  interesting thing I can think of to do.
// @include       *
// @exclude       http://metameso.org:9000/*
// @version       1.0 - Initial Version - Sunday February 15, 2009
// ==/UserScript==

(function() {

  // Add a new Global Key Listener for `esc`
  document.addEventListener("keypress", function(e) {
    if(!e) e=window.event;
    var key = e.keyCode ? e.keyCode : e.which;
    if ( key === 27 ) {
      alert("You're browsing \nhttp://"+window.location);
      }
  }, true);

// Margin, top, left, width and height center the iframe horizontally and vertically:
 var css = 'position:fixed; z-index:9999; border:1px solid black; ' +
          'left:50%; width:50%; height:100%;';

// position:fixed means stay fixed even when the page scrolls. z-index keeps your iframe on top.
// The remainder of the line smacks the panel into the bottom left corner, out of your way.
// Overflow (in combination with the setTimeout) ensures the iframe fits your entire panel.
// var css = 'position:fixed; z-index:9999; bottom:0; left:0; border:0; margin:0; padding:0; ' +
//          'overflow:hidden;'

var iframe = document.createElement('iframe');
iframe.setAttribute('style', css);

// The about:blank page becomes a blank(!) canvas to modify
iframe.src = 'about:blank';

document.body.appendChild(iframe);

// Make sure Firefox initializes the DOM before we try to use it.
iframe.addEventListener("load", function() {
    var doc = iframe.contentDocument;
    doc.body.style.background = 'white';
    doc.body.innerHTML = 'Foo!';
    iframe.style.width = doc.body.offsetWidth + "px";
    iframe.style.height = doc.body.offsetHeight + "px";
}, false);


})();

