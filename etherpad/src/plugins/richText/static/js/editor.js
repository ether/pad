var richTextClient = {
    localSelection :{
        selStart : [],
        selEnd   : [],
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
                    ace.ace_performSelectionChange([rep.selStart[0],rep.selStart[1]-1], rep.selStart, false);
                    ace.ace_performDocumentApplyAttributesToRange(rep.selStart, rep.selEnd,
                           [["imgSrc", value.url]]);
                }, "insertImage", true);
                break; 
              case "link":
                rtLinkDlg.show();
                break;
              case "insertLink":
                alert(value.url+ " : " + value.text);
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
                    style[ pair[0] ] = pair[1];
                }
            }
        }
        return style;
    },
    /**
    * convert font-size -> fontSize
    **/
    formatStyleName : function(name){
       return (name || "").replace(/\-(.)?/,function(_, s){return s.toUpperCase()})
    },
    collectContent : function(args){
        if(args.tname){
            var style = richTextClient.evalStyleString(args.styl);
            if( "span" == args.tname.toLowerCase()){
                if(!style) return ;
                var lists = ["color", "font-family", "font-size", "background-color"], name;
                for(var i = 0, len = lists.length; i < len; i++){
                    name = lists[i];
                    if(style[name]){
                        args.cc.doAttrib(args.state, richTextClient.formatStyleName(name), style[name]);
                    }
                }
            } else if( "div" == args.tname.toLowerCase() &&  -1 == (args.cls || "").indexOf("ace-line")){
                if(!style) return ;
                var lists = ["text-align"], name;
                for(var i = 0, len = lists.length; i < len; i++){
                    name = lists[i];
                    if(style[name]){
                        args.cc.doLineAttrib(args.state, richTextClient.formatStyleName(name), style[name]);
                    }
                }
            }
        }
    },
    parseCommand : function(args){
        if(!args) return ;
        var attributes = args.attributes;
        var attStr = "", noderef = [], blockref = [], style = {}, temp = {},
               cmd = "", value = "", extraOpenTags = "", extraCloseTags = "";
        if(attributes && attributes.length){
            for(var i = 0, len = attributes.length; i < len; i++){
                var pool = attributes[i];
                switch(pool[0]){
                    case "imgSrc":
                        extraOpenTags += "<img src=" + pool[1] + " />"; 
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
        }
        return [ {attStr : attStr, noderef: noderef, blockref : blockref, extraOpenTags : extraOpenTags, extraCloseTags : extraCloseTags}];
    }
}
