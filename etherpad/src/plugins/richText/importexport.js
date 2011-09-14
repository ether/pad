jimport("java.lang.System.out.println");

/**
 * conver fontSize to font-size
 */
function getCSSRuleName(name){
  return (name || "").replace(/([A-Z])/g,function(_, s){return '-' + s.toLowerCase()})
}

/**
 * convert font-size -> fontSize
 **/
function getJsRuleName(name){
   return (name || "").replace(/\-(.)?/g,function(_, s){return s.toUpperCase()})
}

var spanStyle = ['color', 'backgroundColor', 'fontSize',
    'fontFamily', 'width', 'height'];

function isSpanStyle(name){
  for(var i = 0, len = spanStyle.length; i < len; i++){
    if (name == spanStyle[i]){
      return true;
    }
  }
  return false;
}

var MAX_LIST_LEVEL = 10;
var olRecorder = new Array(MAX_LIST_LEVEL);
var olFlag = false;

function getOlIndex(level){
  if(level >= MAX_LIST_LEVEL || level < 0){
    return 1;
  }
  var index = (olRecorder[level] || 0);
  index ++;
  olRecorder[level] = index;
  for(var i = level + 1; i < MAX_LIST_LEVEL; i++){
    olRecorder[i] = 0;
  }
  return index;
}

function afterExport(){
  for(var i = 0; i < MAX_LIST_LEVEL; i++){
    olRecorder[i] = 0;
  }
}

function exportInlineStyle(args){
  var result = {
    isAceObject    : false,
    extraOpenTags  : "",
    extraCloseTags : ""
  }  
  if (args && args.attributes) {
    var useSpan = false;
    var spanStyleList = {};
    for(var i = 0, len = args.attributes.length; i < len; i++) {
      attribName  = args.attributes[i].name;
      attribValue = args.attributes[i].value;
      if (isSpanStyle(attribName)){
        var cssRuleName = getCSSRuleName(attribName);   
        spanStyleList[cssRuleName] = attribValue;
        useSpan = true;
      } else {
        var attStr = "";
        switch (attribName){
          case "imgSrc":
              result.extraOpenTags += "<img src=\"" + attribValue + "\" />";
              break;
          case "aceObject":
              result.isAceObject = true;
              break;
          default:
              break;
        }
      }
    }
  }
  if (useSpan) {
    var openSpan = "<span style=\"";   
    for (var style in spanStyleList){
      openSpan += style + ":" + spanStyleList[style] + ";";
    }
    openSpan += "\">"
    result.extraOpenTags  += openSpan;
    result.extraCloseTags += "</span>";
  }
  return [result];
}

function exportLineMarkerStyle(args){
  var result = {
    extraOpenTags  : "",
    extraCloseTags : ""
  }  
  if (args && args.attributes) {
    var name = "", value = "", level = 1, index = 1;
    for(var i = 0, len = args.attributes.length; i < len; i++){
      name  = args.attributes[i].name;
      value = args.attributes[i].value;
      println("[Line Style] Name :" + name + " Value : " + value);
      switch(name){
        case "textAlign":
          result.extraOpenTags += "<div style=\"text-align:"+ value + "\">";
          result.extraCloseTags = "</div>" + result.extraCloseTags;
          break;
        case "preDefinedStyle":
          if (value == "blockquote") {
            result.extraOpenTags += "<blockquote class=\"richquotestyle\" " +i //TODO add style sheet support
                  "style=\"background-color: #E5ECF9;border:1px solid gray;margin: 0.5em;\">";
            result.extraCloseTags = "</blockquote>" + result.extraCloseTags;
          } else {
            result.extraOpenTags += "<" + value + ">";
            result.extraCloseTags = "</" + value + ">" + result.extraCloseTags;
          }
          break;
        case "list":
          if(value && value.length > 6){
            level = parseInt(value.substr(6)) || 1;
            println("Get level :" + value.substr(6) + " level : " + level);
          }
          break;
        case "orderedlist":
          index = getOlIndex(level);
          println("Get Index : " + index + " Using Level : " + level);
          for(var step = 1; step < level; step++){
            result.extraOpenTags += "<ol style=\"list-style-type:none;\"><li>";
            result.extraCloseTags += "</li></ol>" + result.extraCloseTags;
          }
          result.extraOpenTags += "<ol start=\"" + index + "\"><li>";
          result.extraCloseTags += "</li></ol>" + result.extraCloseTags;
          break;
        default:
          break;
      }
    }
  }
  return [result];
}

function _trim(str){
  str = str || "";
  var ret = "";
  var start = 0, end = str.length;
  while(" " == str[start] && start < end){
    start ++;
  }
  do{
    end --;
  } while(" " == str[end]&& end >= start)
  ret = str.substr(start, end - start + 1);
  return ret;
}

function evalStyleString (str){
  var style = null;
  if(str && str.length){
      style = {};
      var pairs = str.split(/;|,/);
      for(var i = 0, len = pairs.length; i < len; i++){
          var pair = pairs[i].split(":");
          if(pair && pair.length == 2){
              style[_trim(pair[0])] = _trim(pair[1]);
          }
      }
  }
  return style;
}

var _blockElems = { "div":1, "p":1, "pre":1};
function isBlockElement(tagName) {
  return !!_blockElems[(tagName || "").toLowerCase()];
}

function isStyledBlockElement (tagName){
  var tagLists = ["h1", "h2", "h3", "h4", "h5", "h6", "blockquote"];
  var tagReg = new RegExp(tagLists.join("|"));
  return !! tagReg.test(tagName || "");
}

var importOlLevel = 0;
function collectContentPre(args){
  if(args.tname){
      var style = evalStyleString(args.styl), tname = args.tname.toLowerCase(),
          attribs = args.attribs;
      if("ol" == tname){
        olFlag = true;
        importOlLevel ++;
      } else if("ul" == tname){
        olFlag = false;
      }
      if( "span" == tname){
          if(!style) return ;
          var lists = ["color", "font-family", "font-size", "background-color"], name;
          for(var i = 0, len = lists.length; i < len; i++){
              name = lists[i];
              if(style[name]){
                  args.cc.doAttrib(args.state, getJsRuleName(name), style[name]);
              }
          }
      } else if(isBlockElement(tname)){
          if(style) {
            var lists = ["text-align"], name;
            for(var i = 0, len = lists.length; i < len; i++){
              name = lists[i];
              if(style[name]){
                  args.cc.doLineAttrib(args.state, getJsRuleName(name), style[name]);
              }
            }
          }
          var attLists = [{attr : "align", command : "text-align"}]; //trick for soffice
          for(var i = 0, len = attLists.length; i < len; i++){
            var attr = attLists[i].attr;
            var cmd  = attLists[i].command;
            var val  = attribs[attr] || attribs[ attr.toUpperCase()];
            if(val){
              args.cc.doLineAttrib(args.state, getJsRuleName(cmd), val.toLowerCase());
            }
          }
      } else if("img" == tname){
          if(style){
              var lists = ["height", "width"], name;
              for(var i = 0, len = lists.length; i < len; i++){
                  name = lists[i];
                  if(style[name]){
                      args.cc.doAttrib(args.state, getJsRuleName(name), style[name]);
                  }
              }
          }
          args.cc.doObjAttrib(args.state, "imgSrc", attribs.src);
      } else if("li" == tname && olFlag){
          args.cc.doLineAttrib(args.state, "orderedlist", "true");
//          args.cc.doAttrib(args.state, "list", "bullet" + importOlLevel);
      } else if(isStyledBlockElement(tname)){ //predefined style
          args.cc.doLineAttrib(args.state, "preDefinedStyle", tname);
      } else if("a" == tname && attribs.href){
          args.cc.doAttrib(args.state, "link", attribs.href);
      }
  }
}

function collectContentPost(args){
  if("ol" == args.tname){
      importOlLevel --;
  }
}
