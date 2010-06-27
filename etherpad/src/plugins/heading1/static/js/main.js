function init() {
  this.hooks = ['aceAttribsToClasses', 'aceCreateDomLine'];
  this.aceAttribsToClasses = aceAttribsToClasses;
  this.aceCreateDomLine = aceCreateDomLine;
}

function aceAttribsToClasses(args) {
  if (args.key == 'heading1' && args.value != "")
    return ["heading1:" + args.key + ":" + args.value];
}

function aceCreateDomLine(args) {
  if (args.cls.indexOf('heading1:') >= 0) {
    cls = args.cls.replace(/(^| )heading1:(\S+)/g, function(x0, space, padtagsearch) { return ''; });
    return [{cls: cls, extraOpenTags: '<h1>', extraCloseTags: '</h1>'}];
  }
}

function heading1clicked(event) {
  padeditor.ace.callWithAce(function (ace) {
  ace.ace_toggleAttributeOnSelection("heading1");
 }, "heading1", true);

}

/* used on the client side only */
heading1 = new init();
