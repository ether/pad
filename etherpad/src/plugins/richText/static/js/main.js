if(typeof dojo != "undefined"){
    dojo.require("dijit.form.Button");
    dojo.require("dijit.Menu");
    dojo.require("dijit.ColorPalette");
    dojo.require('dijit.form.Select');
    dojo.require("dijit.Dialog");
    dojo.require("dijit.form.ValidationTextBox");
}



function colorPalette(name, img, callback){ 
	//html && dom string must follow a sepecify rule see the html content in the test file
	var Palette = new dijit.ColorPalette({
                palette: "7x10",
                onChange: function(val) {
        		dojo.style(name + "ColorIndicator", "backgroundColor", val);
       	        	Button.set("value", val);
	        	if(callback){
			   callback(val);
			}
                }
              },
              name + "CPplaceHolder");

        var Button = new dijit.form.ComboButton({
               label: "<span style='position:relative'><img src=' " + img + "' />\
          			<span id='"+ name +"ColorIndicator'style='background-color:#000;display:block;\
		        	 position:absolute;height:5px; top:11px; width:100%'>&nbsp;</span></span>",
               name: name + "cp",
    	       value : "#000000",
               dropDown: Palette,
		       onClick : function(){
			       if(callback){
        			   callback(this.get("value"));
		           }
		       },
               id: name + "cpButton"
        });
        dojo.byId(name + "ColorPalette").appendChild(Button.domNode);
}

function buildColorPalette(){ //build text-color and background-color palette
	colorPalette("text", "/static/img/plugins/richText/textcolor.gif", function(val){
          richTextexecCommand("color", val);
     });
	colorPalette("bg", "/static/img/plugins/richText/bgcolor.gif", function(val){
          richTextexecCommand("backgroundColor", val);
     });
}

function styleMenuList(name, style, values, callback, selectedIndex){
    var menu = new dijit.Menu({
         //  style: "display: none; width:100px",
         style: { width: '100px', overflow:"hidden" },
         forceWidth : true
    });
	selectedIndex = selectedIndex || 0;
	var strList = [
	   "<span style='",
	   style,
	   ":",
	   "", //value
	   "'>",
	   "", //value
	   "</span>" 		  
	];
	for(var i = 0, len = values.length; i < len; i++){
		strList[3] = strList[5] = values[i];
        var menuItem = new dijit.MenuItem({
	          label : strList.join(""), 
	 	      value : values[i],
              onClick: function() {
		      var val = this.get("value");		
		    	   button.set({"label":val, "value": val});
			       if(callback){
				        callback(val);
    			   }
	          }
       });
       menu.addChild(menuItem);
	}

    var button = new dijit.form.DropDownButton({
          label: values[selectedIndex],
	      value: values[selectedIndex], 
          name: name + "menulist",
          dropDown: menu,
          id: name + "menu"
    });
    dojo.byId(name + "placeholder").appendChild(button.domNode);
} 
	
function familyLists(values, callback, selectedIndex){
	var strList = [
	   "<span style='",
	   "font-family",
	   ":",
	   "", //value
	   "'>",
	   "", //value
	   "</span>" 		  
	];
	var opts = [];
	selectedIndex = selectedIndex || 0;
	for(var i = 0, len = values.length; i < len; i++){
		strList[3] = strList[5] = values[i];
		opts.push({
			label : strList.join(""),
			value : values[i]	
		});
	}	
	strList[3] = strList[5] = values[selectedIndex];
	var select = new dijit.form.Select({
	    id: 'fontfamilysetter',
		label : strList.join(""),
		value : values[selectedIndex],
	    style: { width: '130px', overflow:"hidden" },
	    forceWidth : true,
		onChange : function(val){
			if(callback){
			  callback(val);
			}
		},
	    options: opts
    }).placeAt("fontfamilyplaceholder");
}


function preDefinedList(lists, name, selectedIndex, callback){
    var menu = new dijit.Menu({
         //  style: "display: none; width:100px",
         style: { width: '100px', overflow:"hidden" },
         forceWidth : true
    });
	selectedIndex = selectedIndex || 0;

	var strList = [
	   "<",
       "", //tagname
	   " class='",
	   "", //classname
	   "'>",
	   "", //title
	   "</",
       "", //tagname
       ">" 		  
	];
	selectedIndex = selectedIndex || 0;
    var item = null;
	for(var i = 0, len = lists.length; i < len; i++){
        item = lists[i]; 
        strList[1] = strList[7] = item.tagName;
        strList[3] = item.className;
        strList[5] = item.title;
        var menuItem = new dijit.MenuItem({
	          label : strList.join(""), 
              title : item.title,
	 	      value : item.value,
              onClick: function() {
    		      var val = this.get("value"), title = this.get("title");		
	    	      button.set({"label": title, "value": val});
		    	  if(callback){
			         callback(val);
    			  }
    	      }
        });
        menu.addChild(menuItem);
	}	

    var button = new dijit.form.DropDownButton({
          label: lists[selectedIndex].title,
	      value: lists[selectedIndex].value, 
          name: name + "menulist",
          dropDown: menu,
          id: name + "menu"
    });
    dojo.byId(name + "placeholder").appendChild(button.domNode);
}

function buildFontStyle(){
	var sizes = ["10px", "12px", "16px", 
			"18px", "24px", "32px", "48px"];
	var families = ["Arial", "Arial Black", "Comic Sans MS", 
			"Georgia", "Times New Roman", "\u5b8b\u4f53", "\u6977\u4f53"]; 
    var preDefined = [
            {title:"Heading 1", value : "h1", tagName : "h1", className :""},
            {title:"Heading 2", value : "h2", tagName : "h2", className :""},
            {title:"Heading 3", value : "h3", tagName : "h3", className :""},
            {title:"Heading 4", value : "h4", tagName : "h4", className :""},
            {title:"Heading 5", value : "h5", tagName : "h5", className :""},
            {title:"Heading 6", value : "h6", tagName : "h6", className :""},
            {title:"Block Quote", value : "blockquote", tagName : "blockquote", className :"richquotestyle"},
            {title:"Content", value : "content", tagName : "span", className :""}
    ];

    styleMenuList("fontsize", "font-size", sizes, function(val){
          richTextexecCommand("fontSize", val);
    });	
	//styleMenuList("fontfamily", "font-family",
	//families, function(val){alert(val)}); //how to set fix width in dojo?
	familyLists(families, function(val){
          richTextexecCommand("fontFamily", val);
    });

    preDefinedList(preDefined, "preDefinedStyle", 0, function(val){
          richTextexecCommand("preDefinedStyle", val);
    });
}

function isFunction(func){
     return (func instanceof Function);
}

var fsxGetUniqueString=(function (){
        var seed_map=[];
        return function(seed){
            if(!seed_map[seed])
                seed_map[seed]=1;
             return seed+seed_map[seed]++;
       }
})();

function fsxGetUniqueID(seed,_doc){
     seed=seed||"FSX_UNIQUE_ID_";
     var id=fsxGetUniqueString(seed);
     var doc=_doc||document;
     while(doc.getElementById(id)){
         id=fsxGetUniqueString(seed);
     }
    return id;
}

var confirmDialog = (function(){
    
    function buildHolderString(id, lists){
        var elem = document.createElement("div");
        elem.id = id;
        elem.style.display = "none";
        var str = "";
        for(var i = 0, len = lists.length; i < len; i++){
            str += "<div style='margin-bottom:0.2em; font-size:1.2em'><label>" 
                    + lists[i].name + " : </label><span id=" + id + "input_" 
                    + i + "></span></div>" 
        }
        var foot = "<div id="+ id+"_bar" +" class=dijitDialogPaneActionBar></div>";
        str += foot;
        elem.innerHTML = str;
        document.body.appendChild(elem);
    }    

    return function(title, lists, config){
       config = config || {};
       this.id = fsxGetUniqueID("ep_dialog_");
       buildHolderString(this.id, lists);
       this.dlg = new dijit.Dialog({
                     title: "Insert Image",
                     style: "width: 300px"
        }, this.id); 
        this.buttons = [];
        for(var i = 0, len = lists.length; i < len; i++){
            var btn = new dijit.form.ValidationTextBox({
                       required : lists[i].required,
                       width : (lists[i].width || 240) 
            }).placeAt(this.id + "input_" + i);
            this.buttons.push(btn);
        }
        var self = this;
        var IDOK = new dijit.form.Button({
               label: "OK",
               onClick: function() {
                   var ret = {}; 
                   var btns = self.buttons;
                   for(var i = 0, len = btns.length; i < len; i++){
                        if(btns[i].isValid()){
                            ret[lists[i].value] = btns[i].get("value");
                        }else{
                            return;
                        }
                    } 
                    self.dlg.hide();
                    if(isFunction(config.onOk)){
                        config.onOk(ret);
                    }
               }
        }).placeAt(this.id + "_bar"); 
        var IDCancel = new dijit.form.Button({
               label: "Cancel",
               onClick: function() {
                    self.dlg.hide();
               }
        }).placeAt(this.id + "_bar");
    }
})();

var rtImgDlg, rtLinkDlg;

function buildDialogs(){
     var cDlg = new confirmDialog("Insert Image",
                     [{ name : "Image URL", value : "url", required : true}], 
                     {
                           onOk: function(ret){
                                richTextexecCommand("insertImage", ret); 
                           }
                     }
     );
     rtImgDlg = cDlg.dlg;
     
     cDlg = new confirmDialog("Insert Link",
                     [
                        { name : "URL ", value : "url", required : true},
                        { name : "Display text", value : "text"}
                     ], 
                     {
                           onOk: function(ret){
                                richTextexecCommand("insertLink", ret); 
                           }
                     }
     );
     rtLinkDlg = cDlg.dlg; 
}	

if(typeof dojo != "undefined"){
    dojo.addOnLoad(function() {
        buildColorPalette();
        buildFontStyle();
    });
}
if(typeof window == "object"){
var richTextClient = {
    localSelection :{
        selStart : [],
        selEnd   : []
    },
    recordSelection : function(start, end){
        if(!start || !end) return;
        this.localSelection.selStart = start.concat([]);
        this.localSelection.selEnd   = end.concat([]);
    },
    getRecordSelection : function(){
        return this.localSelection;
    },
    execCommand : function(cmd, value){
         switch(cmd){
             case "textAlign":
                padeditor.ace.callWithAce(function (ace) {
                    ace.ace_toggleAttributeOnLine(cmd, value);
                }, cmd, true);
                break;
              case "image":
                padeditor.ace.callWithAce(function (ace) {
                     var rep = ace.ace_getRep();
                     richTextClient.recordSelection(rep.selStart, rep.selEnd);
                }, "image", true);
                rtImgDlg.show();
                break;
              case "insertImage":
                padeditor.ace.callWithAce(function (ace) {
                    var rep;
                    if(!value || !value.url) return;
                    rep = richTextClient.getRecordSelection();
                    ace.ace_replaceRange(rep.selStart, rep.selEnd, ace.objMarker);
                    ace.ace_performSelectionChange([rep.selStart[0],rep.selStart[1]], rep.selStart, false);
                    ace.ace_performDocumentApplyAttributesToRange(rep.selStart, [rep.selStart[0],rep.selStart[1] + 1],
                           [["aceObject", "true"],["imgSrc", value.url]]);
                }, "insertImage", true);
                return; 
              case "orderedlist":
                padeditor.ace.callWithAce(function(ace){
                    ace.ace_toggleAttributeOnLine(cmd, "true");
                }, "orderedlist", true); 
                break;
              case "Eraser":
                padeditor.ace.callWithAce(function(ace){
                    ace.ace_eraseTextAttributeOnSelection();
                }, "Eraser", true); 
                break;
              case "link":
                padeditor.ace.callWithAce(function (ace) {
                     var rep = ace.ace_getRep();
                     richTextClient.recordSelection(rep.selStart, rep.selEnd);
                }, "link", true);
                rtLinkDlg.show();
                break;
              case "insertLink":
                 padeditor.ace.callWithAce(function (ace) {
                    var rep;
                    if(!value || !value.url || !value.text) return;
                    rep = richTextClient.getRecordSelection();
                    ace.ace_replaceRange(rep.selStart, rep.selEnd, value.text);
                    ace.ace_performSelectionChange([rep.selStart[0],rep.selStart[1]], rep.selStart, false);
                    ace.ace_performDocumentApplyAttributesToRange(rep.selStart, [rep.selStart[0],rep.selStart[1] + value.text.length],
                           [["link", value.url]]);
                }, "insertLink", true);
                break;
              case "unlink":
                padeditor.ace.callWithAce(function(ace){
                    ace.ace_toggleAttributeOnSelection("link", "");
                }, "unlink", true); 
                break;
              case "preDefinedStyle":
                 padeditor.ace.callWithAce(function (ace) {
                    if(value == "content"){
                        value = "";
                    }
                    ace.ace_toggleAttributeOnLine(cmd, value);  
                }, cmd, true);
                break; 
              default:
                padeditor.ace.callWithAce(function (ace) {
                    ace.ace_toggleAttributeOnSelection(cmd, value);
                 }, cmd, true);
         }
         padeditor.ace.focus();
    },
    joinStyle : function(style){
        var str = "";
        for(var i in style){
            str += i + ":" + style[i] + ";";
        }
        if(str.length){
            str = "style='" + str + "'"; 
        }
        return str;
    }, 
    evalStyleString : function(str){
        var style = null;
        if(str && str.length){
            style = {};
            var pairs = str.split(/;|,/);
            for(var i = 0, len = pairs.length; i < len; i++){
                var pair = pairs[i].split(":");
                if(pair && pair.length == 2){
                    style[ pair[0].trim() ] = pair[1].trim();
                }
            }
        }
        return style;
    },
    /**
    * convert font-size -> fontSize
    **/
    formatStyleName : function(name){
       return (name || "").replace(/\-(.)?/g,function(_, s){return s.toUpperCase()})
    },
    isStyledBlockElement : function(tagName){
        var tagLists = ["h1", "h2", "h3", "h4", "h5", "h6", "blockquote"];
        var tagReg = new RegExp(tagLists.join("|"));
        return !! tagReg.test(tagName || "");
    },
    collectContent : function(args){
        if(args.tname){
            var style = richTextClient.evalStyleString(args.styl), tname = args.tname.toLowerCase(),
                attribs = args.attribs;
            if( "span" == tname){
                if(!style) return ;
                var lists = ["color", "font-family", "font-size", "background-color"], name;
                for(var i = 0, len = lists.length; i < len; i++){
                    name = lists[i];
                    if(style[name]){
                        args.cc.doAttrib(args.state, richTextClient.formatStyleName(name), style[name]);
                    }
                }
            } else if( "div" == tname &&  -1 == (args.cls || "").indexOf("ace-line")){
                if(!style) return ;
                var lists = ["text-align"], name;
                for(var i = 0, len = lists.length; i < len; i++){
                    name = lists[i];
                    if(style[name]){
                        args.cc.doLineAttrib(args.state, richTextClient.formatStyleName(name), style[name]);
                    }
                }
            } else if("img" == tname){
                if(style){
                    var lists = ["height", "width"], name;
                    for(var i = 0, len = lists.length; i < len; i++){
                        name = lists[i];
                        if(style[name]){
                            args.cc.doAttrib(args.state, richTextClient.formatStyleName(name), style[name]);
                        }
                    }
                }
                args.cc.doObjAttrib(args.state, "imgSrc", attribs.src);
            } else if("ol" == tname && /\bace\-orderedlist\b/.exec(args.cls)){
                args.cc.doLineAttrib(args.state, "orderedlist", "true");
            } else if(richTextClient.isStyledBlockElement(tname)){ //predefined style
                args.cc.doLineAttrib(args.state, "preDefinedStyle", tname);
            } else if("a" == tname && attribs.href){
                args.cc.doAttrib(args.state, "link", attribs.href);
            } 
        }
    },
    parseCommand : function(args){
        if(!args) return ;
        var attributes = args.attributes;
        var attStr = "", noderef = [], blockref = [], style = {}, temp = {},
               cmd = "", value = "", extraOpenTags = "", extraCloseTags = "", cls = "", redirect = false;
        if(attributes && attributes.length){
            for(var i = 0, len = attributes.length; i < len; i++){
                var pool = attributes[i];
                switch(pool[0]){
                    case "imgSrc":
                        extraOpenTags += "<img ondragend='customDragEnd()' src=" + pool[1] + " "; /*manually fire dragend event on chrome*/ 
                        cls="ace-placeholder";
                        redirect = true;
                        break;
                    case "color":
                        style.color = pool[1];
                        break;
                    case "backgroundColor":
                        style["background-color"] = pool[1];
                        break;
                    case "fontSize":
                        style["font-size"] = pool[1];
                        break;
                    case "fontFamily":
                        style["font-family"] = pool[1];
                        break;
                    case "width":
                        style["width"] = pool[1];
                        break;
                    case "height":
                        style["height"] = pool[1]; 
                        break;
                    case "link":
                        extraOpenTags += "<a href='" + pool[1] +"' >"; /*manually fire dragend event on chrome*/ 
                        extraCloseTags = "</a>" + extraCloseTags;
                        break;
                    case "preDefinedStyle":
                        temp = {
                            tag : "div", 
                            className : "", //class can't be used as attribute in safari
                            attrs : {
                            }
                        };
                        switch(pool[1]){
                            case "h1":
                            case "h2":
                            case "h3":
                            case "h4":
                            case "h5":
                            case "h6":
                                temp.tag = pool[1]; 
                                break;
                            case "blockquote":
                                temp.tag = "blockquote";
                                temp.className = "richquotestyle";
                                break;
                            default:
                                tep = {};
                                
                        }
                        blockref.push(temp);
                        break;
                    case "textAlign":
                        temp = {
                            tag : "div", 
                            attrs : {
                                style : {}
                            }
                        };
                        temp.attrs.style = {"text-align" : pool[1]};
                        blockref.push(temp);
                        break;
                } 
            }
            var styleStr = richTextClient.joinStyle(style);
            attStr += styleStr;
            if(redirect){ //need to update
                extraOpenTags += styleStr + " />";
            }
        }
        return [ {attStr : attStr, noderef: noderef, blockref : blockref, 
                 cls : cls, extraOpenTags : extraOpenTags, extraCloseTags : extraCloseTags}];
    }
}
}
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
