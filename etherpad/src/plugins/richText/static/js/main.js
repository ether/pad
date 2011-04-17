function richTextInit() {
  this.hooks = ['aceAttribsToClasses', 'aceCreateDomLine',
	 'collectContentPre', 'collectContentPost', 'aceCreateStructDomLine',
     'aceInitInnerdocbodyHead'];
  this.aceAttribsToClasses = aceAttribsToClasses;
  this.aceCreateDomLine = aceCreateDomLine;
  this.collectContentPre = collectContentPre;
  this.collectContentPost = collectContentPost;
  this.aceCreateStructDomLine = aceCreateStructDomLine;
  this.aceInitInnerdocbodyHead = aceInitInnerdocbodyHead;
}
if(typeof richTextClient == "undefined"){
    var richTextClient ={
        execCommand    : function(){},
        parseCommand   : function(){},
        collectContent : function(){}
    }
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

function aceInitInnerdocbodyHead(args){
    args.iframeHTML.push('\'<link rel="stylesheet" type="text/css" href="/static/css/plugins/richText/richtext.css"/>\'');
}

function collectContentPost(args) {
//  if (args.tname == "h1")
//    args.cc.startNewLine(args.state);
}

/* used on the client side only */
richText = new richTextInit();
