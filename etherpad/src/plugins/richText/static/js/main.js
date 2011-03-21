function richTextInit() {
  this.hooks = ['aceAttribsToClasses', 'aceCreateDomLine',
	 'collectContentPre', 'collectContentPost', 'aceCreateStructDomLine'];
  this.aceAttribsToClasses = aceAttribsToClasses;
  this.aceCreateDomLine = aceCreateDomLine;
  this.collectContentPre = collectContentPre;
  this.collectContentPost = collectContentPost;
  this.aceCreateStructDomLine = aceCreateStructDomLine;
}


//rich text command entry
function richTextexecCommand(cmd, value){
    richTextClient.execCommand(cmd, value);
}

function aceAttribsToClasses(args) {
}

function aceCreateDomLine(args) {
}

function aceCreateStructDomLine(args){
    return richTextClient.parseCommand(args);
}

function collectContentPre(args) {
// if (args.tname == "h1") {
//   args.cc.doAttrib(args.state, "heading1");
// }
}

function collectContentPost(args) {
//  if (args.tname == "h1")
//    args.cc.startNewLine(args.state);
}

/* used on the client side only */
richText = new richTextInit();
