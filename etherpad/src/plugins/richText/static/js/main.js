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
    switch(cmd){
        case "justifyleft":
        case "justifyright":
        case "justifycenter":
        case "justifyjustify":
            value = cmd.replace("justify", "");             
            cmd = "textAlign";
            break; 
    }
    richTextClient.execCommand(cmd, value);
}

function aceAttribsToClasses(args) {
    if("orderedlist" == args.key && args.value){
        return ["ace-orderedlist"];
    }
}

function aceCreateDomLine(args) {
}

function aceCreateStructDomLine(args){
    return richTextClient.parseCommand(args);
}

function collectContentPre(args) {
    return richTextClient.collectContent(args);
}

function collectContentPost(args) {
//  if (args.tname == "h1")
//    args.cc.startNewLine(args.state);
}

/* used on the client side only */
richText = new richTextInit();
