/* This is similar to the server-side main.js.  We don't
   need to add handlePath here, because handling the path
   happens on the server side.  Also (for some reason) we
   don't need any importing here; I guess the three
   client-side hooks are defined in this file, is why. */

function init() {
  this.hooks = ['aceInitInnerdocbodyHead', 'aceGetFilterStack', 'aceCreateDomLine'];
  this.aceInitInnerdocbodyHead = aceInitInnerdocbodyHead;
  this.aceGetFilterStack = aceGetFilterStack;
  this.aceCreateDomLine = aceCreateDomLine;
}

/* All this iframe stuff is obviously client-side. */

function aceInitInnerdocbodyHead(args) {
  console.log("Hi!");
  args.iframeHTML.push('\'<link rel="stylesheet" type="text/css" href="/static/css/plugins/wikiStyleLinks/pad.css"/>\'');
}

/* aceGetFilterStack will run both server- and client-side. */

/* Returns a set of functions that filters the pad content
   [[using a regexp]].  The args parameter here has
   methods, including linestylefilter.getRegexpFilter. */

/* Cf. ~/etherpadding/pad-may-9/infrastructure/ace/www/linestylefilter.js */

/* getRegexpFilter is a function that adds the 'wikilink'
   attribute to everything that matches the regexp. */

/* Note, we may have to be a bit careful because we might
   see [[this is just designed to confuse #people]] or
   [[have you seen http://doc.etherpad.org lately?]] or
   even worse, #what-is-this-[[supposed]]-to-be?  

   On the other hand, to keep the code simple and modular,
   I think what's going to happen is that the "outer"
   syntax will dominate.

   And what about things like [[this one is [[tricky]]
   or [[this one is [[tricky]]]] ?

   It seems like the 1st will work as a link, and the 2nd
   will link to the same thing.  I guess that's fine.*/

 /* Right now, the regexp is supposed to say:
    1. find [[
    2. followed by anything that isn't ]
    3. and then by ]]
    NEXT PASS THROUGH THE CODE:
    -- and put a bracket around that 2nd bit
    -- then indicate the 1st bracketed expression. */

function aceGetFilterStack(args) {
  return [
    args.linestylefilter.getRegexpFilter(
      new RegExp("\\[\\[[^\\[\\]]*]]", "g"), 'wikilink')
  ];
}

/* This is where we actually do things.  Now that
   different ranges of text are labelled in the document,
   you look through the text, you add HTML around the
   labelled bits.  (This works on an abstract, object
   oriented version of the pad content, whereas the former
   works on a SIMPLE version of the text -- a string --
   and returns a string with properties!)  */

/* args.cls is a list of attributes. 
   args.cls.indexOf >= 0 means that the thing is contained in the list*/

function aceCreateDomLine(args) {
  if (args.cls.indexOf('wikilink') >= 0) {
    var href;
    /* cls contains the name of a pad that's being linked to --
       We replace it with the word "wikilink" so that args.cls 
       is a list of valid CSS classes. 
       cls = "whatever-class other-class wikilink:[[whatever goes here]] yet-another-class" */
    cls = args.cls.replace(/wikilink:\[\[([^\[\]]*)]]/g, function(x0,linktext) {
      /*At the same time we convert the name of the pad
        into a proper URL.*/
      href = "/" + linktext.replace(/ /g, '-');
      return "wikilink";
    });

   return [{
     cls: cls,

     /* Not sure about this replacement relative to the
        other one above... */

     extraOpenTags: '<a href="' + href + '">',
     extraCloseTags: '</a>'}];
  }
}

/* used on the client side only; something similar needed
   in every client-side plugin to get it to go*/

wikiStyleLinks = new init();
