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
    var name = "", value = "";
    for(var i = 0, len = args.attributes.length; i < len; i++){
      name  = args.attributes[i].name;
      value = args.attributes[i].value;
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

function isStyledBlockElement (tagName){
  var tagLists = ["h1", "h2", "h3", "h4", "h5", "h6", "blockquote"];
  var tagReg = new RegExp(tagLists.join("|"));
  return !! tagReg.test(tagName || "");
}

function collectContentPre(args){
  if(args.tname){
      var style = evalStyleString(args.styl), tname = args.tname.toLowerCase(),
          attribs = args.attribs;
      if( "span" == tname){
          if(!style) return ;
          var lists = ["color", "font-family", "font-size", "background-color"], name;
          for(var i = 0, len = lists.length; i < len; i++){
              name = lists[i];
              if(style[name]){
                  args.cc.doAttrib(args.state, getJsRuleName(name), style[name]);
              }
          }
      } else if( "div" == tname &&  -1 == (args.cls || "").indexOf("ace-line")){
          if(!style) return ;
          var lists = ["text-align"], name;
          for(var i = 0, len = lists.length; i < len; i++){
              name = lists[i];
              if(style[name]){
                  args.cc.doLineAttrib(args.state, getJsRuleName(name), style[name]);
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
      } else if("ol" == tname && /\bace\-orderedlist\b/.exec(args.cls)){
          args.cc.doLineAttrib(args.state, "orderedlist", "true");
      } else if(isStyledBlockElement(tname)){ //predefined style
          args.cc.doLineAttrib(args.state, "preDefinedStyle", tname);
      } else if("a" == tname && attribs.href){
          args.cc.doAttrib(args.state, "link", attribs.href);
      }
  }
}

function collectContentPost(){

}
