
function richTextInit() {
  this.hooks = ['aceAttribsToClasses', 'aceCreateDomLine',
	 'collectContentPre', 'collectContentPost'];
  this.aceAttribsToClasses = aceAttribsToClasses;
  this.aceCreateDomLine = aceCreateDomLine;
  this.collectContentPre = collectContentPre;
  this.collectContentPost = collectContentPost;
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
    rep = ace.ace_getRep();
    ace.ace_toggleAttributeOnSelection("heading1");
    ace.ace_replaceRange(rep.selStart, rep.selStart, "\n");
    ace.ace_replaceRange(rep.selEnd, rep.selEnd, "\n");
  }, "heading1", true);
}

function collectContentPre(args) {
  if (args.tname == "h1") {
    args.cc.doAttrib(args.state, "heading1");
  }
}

function collectContentPost(args) {
  if (args.tname == "h1")
    args.cc.startNewLine(args.state);
}

/* used on the client side only */
richText = new richTextInit();


